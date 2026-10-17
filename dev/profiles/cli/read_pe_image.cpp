#include "read_pe_image.h"

namespace profile_development {
template<class T> const T* at(const std::vector<std::uint8_t>& bytes, std::size_t offset) {
    if (offset > bytes.size() || sizeof(T) > bytes.size() - offset) return nullptr;
    return reinterpret_cast<const T*>(bytes.data() + offset);
}

std::optional<PeImage> read_pe(const fs::path& path, std::string& error) {
    PeImage image;
    std::ifstream input(path, std::ios::binary);
    if (!input) { error = "cannot-open-executable"; return std::nullopt; }
    input.seekg(0, std::ios::end);
    const auto length = input.tellg();
    if (length < 4096 || length > static_cast<std::streamoff>(2ull << 30)) {
        error = "invalid-executable-size"; return std::nullopt;
    }
    image.bytes.resize(static_cast<std::size_t>(length));
    input.seekg(0);
    if (!input.read(reinterpret_cast<char*>(image.bytes.data()), static_cast<std::streamsize>(image.bytes.size()))) {
        error = "cannot-read-executable"; return std::nullopt;
    }
    const auto dos = at<IMAGE_DOS_HEADER>(image.bytes, 0);
    if (!dos || dos->e_magic != IMAGE_DOS_SIGNATURE || dos->e_lfanew < 0) {
        error = "invalid-dos-header"; return std::nullopt;
    }
    const auto nt_offset = static_cast<std::size_t>(dos->e_lfanew);
    const auto signature = at<DWORD>(image.bytes, nt_offset);
    const auto file = at<IMAGE_FILE_HEADER>(image.bytes, nt_offset + sizeof(DWORD));
    if (!signature || *signature != IMAGE_NT_SIGNATURE || !file || file->SizeOfOptionalHeader < sizeof(IMAGE_OPTIONAL_HEADER64)) {
        error = "invalid-pe-header"; return std::nullopt;
    }
    const auto optional = at<IMAGE_OPTIONAL_HEADER64>(image.bytes, nt_offset + sizeof(DWORD) + sizeof(IMAGE_FILE_HEADER));
    if (!optional || optional->Magic != IMAGE_NT_OPTIONAL_HDR64_MAGIC) {
        error = "not-pe32-plus"; return std::nullopt;
    }
    image.timestamp = file->TimeDateStamp;
    image.image_size = optional->SizeOfImage;
    image.checksum = optional->CheckSum;
    image.machine = file->Machine;
    const auto section_offset = nt_offset + sizeof(DWORD) + sizeof(IMAGE_FILE_HEADER) + file->SizeOfOptionalHeader;
    for (unsigned i = 0; i < file->NumberOfSections; ++i) {
        const auto section = at<IMAGE_SECTION_HEADER>(image.bytes, section_offset + i * sizeof(IMAGE_SECTION_HEADER));
        if (!section) { error = "truncated-section-table"; return std::nullopt; }
        std::size_t name_size = 0;
        while (name_size < IMAGE_SIZEOF_SHORT_NAME && section->Name[name_size]) ++name_size;
        image.sections.push_back({std::string(reinterpret_cast<const char*>(section->Name), name_size),
            section->VirtualAddress, section->Misc.VirtualSize, section->PointerToRawData, section->SizeOfRawData,
            section->Characteristics});
    }
    const auto exception_directory = optional->DataDirectory[IMAGE_DIRECTORY_ENTRY_EXCEPTION];
    auto rva_to_raw = [&](std::uint32_t rva, std::size_t required) -> std::optional<std::size_t> {
        for (const auto& section : image.sections) {
            if (rva < section.rva) continue;
            const auto delta = static_cast<std::size_t>(rva - section.rva);
            if (delta > section.raw_size || required > section.raw_size - delta) continue;
            const auto raw = static_cast<std::size_t>(section.raw_offset) + delta;
            if (raw <= image.bytes.size() && required <= image.bytes.size() - raw) return raw;
        }
        return std::nullopt;
    };
    if (exception_directory.VirtualAddress && exception_directory.Size >= 12) {
        const auto count = exception_directory.Size / 12;
        const auto raw = rva_to_raw(exception_directory.VirtualAddress, static_cast<std::size_t>(count) * 12);
        if (raw) {
            for (std::uint32_t i = 0; i < count; ++i) {
                struct RuntimeFunctionRow { std::uint32_t begin, end, unwind; } row{};
                std::memcpy(&row, image.bytes.data() + *raw + static_cast<std::size_t>(i) * 12, sizeof(row));
                if (row.begin < row.end && row.end <= image.image_size)
                    image.runtime_functions.push_back({row.begin, row.end, row.unwind});
            }
        }
    }
    return image;
}

std::string sha256(const fs::path& path) {
    BCRYPT_ALG_HANDLE algorithm{};
    BCRYPT_HASH_HANDLE hash{};
    DWORD object_size{}, digest_size{}, received{};
    if (BCryptOpenAlgorithmProvider(&algorithm, BCRYPT_SHA256_ALGORITHM, nullptr, 0) < 0) return {};
    if (BCryptGetProperty(algorithm, BCRYPT_OBJECT_LENGTH, reinterpret_cast<PUCHAR>(&object_size), sizeof(object_size), &received, 0) < 0 ||
        BCryptGetProperty(algorithm, BCRYPT_HASH_LENGTH, reinterpret_cast<PUCHAR>(&digest_size), sizeof(digest_size), &received, 0) < 0) {
        BCryptCloseAlgorithmProvider(algorithm, 0); return {};
    }
    std::vector<UCHAR> object(object_size), digest(digest_size), chunk(1u << 20);
    if (BCryptCreateHash(algorithm, &hash, object.data(), object_size, nullptr, 0, 0) < 0) {
        BCryptCloseAlgorithmProvider(algorithm, 0); return {};
    }
    std::ifstream input(path, std::ios::binary);
    bool ok = static_cast<bool>(input);
    while (ok && input) {
        input.read(reinterpret_cast<char*>(chunk.data()), static_cast<std::streamsize>(chunk.size()));
        const auto count = input.gcount();
        if (count > 0 && BCryptHashData(hash, chunk.data(), static_cast<ULONG>(count), 0) < 0) ok = false;
    }
    ok = ok && input.eof() && BCryptFinishHash(hash, digest.data(), digest_size, 0) >= 0;
    BCryptDestroyHash(hash);
    BCryptCloseAlgorithmProvider(algorithm, 0);
    if (!ok) return {};
    constexpr char digits[] = "0123456789abcdef";
    std::string result;
    result.reserve(digest.size() * 2);
    for (const auto byte : digest) { result.push_back(digits[byte >> 4]); result.push_back(digits[byte & 15]); }
    return result;
}

json image_report(const fs::path& path, const PeImage& pe) {
    json sections = json::array();
    for (const auto& section : pe.sections) {
        sections.push_back({{"name", section.name}, {"rva", section.rva}, {"virtualSize", section.virtual_size},
            {"rawOffset", section.raw_offset}, {"rawSize", section.raw_size},
            {"executable", (section.characteristics & IMAGE_SCN_MEM_EXECUTE) != 0}});
    }
    return {{"schemaVersion", 1}, {"file", fs::absolute(path).string()}, {"fileSize", pe.bytes.size()},
        {"sha256", sha256(path)}, {"machine", pe.machine}, {"timestamp", pe.timestamp},
        {"imageSize", pe.image_size}, {"checksum", pe.checksum}, {"sections", std::move(sections)},
        {"runtimeFunctionCount", pe.runtime_functions.size()}};
}
}
