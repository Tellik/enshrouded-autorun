#include "profile_cli.h"

namespace profile_development {
bool write_json(const fs::path& path, const json& value) {
    std::error_code ec;
    if (!path.parent_path().empty()) fs::create_directories(path.parent_path(), ec);
    if (ec) { std::cerr << "cannot-create-output-directory: " << ec.message() << '\n'; return false; }
    std::ofstream out(path, std::ios::binary | std::ios::trunc);
    if (!out) { std::cerr << "cannot-write: " << path.string() << '\n'; return false; }
    out << value.dump(2) << '\n';
    return static_cast<bool>(out);
}
}
