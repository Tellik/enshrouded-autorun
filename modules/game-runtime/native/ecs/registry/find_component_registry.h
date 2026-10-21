#pragma once

#include "read_component_registry.h"
#include <windows.h>
#include <psapi.h>
#include <algorithm>
#include <array>
#include <chrono>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <functional>
#include <string>
#include <unordered_set>
#include <utility>
#include <vector>

namespace ComponentRegistry {
// Discovery owns no engine pointers beyond validated, borrowed snapshots. The
// parser supplies the complete type contract, not a list of component indices.
// A discovered registry permits metadata inspection only. It does not select a
// world, establish entity offsets or authorize an engine-thread operation.
class Discovery {
public:
    using Validator = std::function<bool(const Snapshot&, std::string&)>;
    using RegionQuery = std::function<bool(std::uintptr_t, MEMORY_BASIC_INFORMATION&)>;
    // The second argument is an element count, not a byte count.
    using ResidentQuery = std::function<bool(PSAPI_WORKING_SET_EX_INFORMATION*, std::size_t)>;
    struct Progress {
        std::uint64_t scanned_bytes{}, skipped_nonresident_bytes{}, validation_bytes{}, budget_limited_steps{}, completed_passes{};
        std::size_t distinct_registries{}, distinct_catalogs{};
        bool scanning{true}, available{};
        std::string reason{"waiting for parser type contract"};
    };

    void Reset() { *this = Discovery{}; }
    const Progress& Status() const { return progress_; }
    // Inspector diagnostics only. These are copies of structurally validated
    // catalogs, not persistent engine addresses or authority to access entities.
    std::vector<Snapshot> CandidateSnapshots() const {
        std::vector<Snapshot> snapshots;
        snapshots.reserve(candidates_.size());
        for (const auto& candidate : candidates_) snapshots.push_back(candidate.snapshot);
        return snapshots;
    }

    bool Tick(std::uintptr_t image, std::size_t image_size, const Reader& read,
              const Validator& validate, Snapshot& output, const RegionQuery& query = {},
              std::chrono::milliseconds step_budget = std::chrono::milliseconds(8),
              const ResidentQuery& resident_query = {}) {
        using Clock = std::chrono::steady_clock;
        // Remote ReadProcessMemory has a different cost from an in-process copy.
        // The external inspector may use a longer step. Production keeps 8 ms.
        step_budget = (std::clamp)(step_budget, std::chrono::milliseconds(1), std::chrono::milliseconds(250));
        const auto deadline = Clock::now() + step_budget;
        if (progress_.scanning && progress_.completed_passes == 0)
            progress_.reason = "scanning resident-private-memory for component registry metadata";
        std::size_t remaining = 2 * 1024 * 1024;
        std::size_t inspected_headers{};
        const auto now = GetTickCount64();
        std::size_t validation_bytes{};
        bool validation_limited{};
        const Reader bounded = [&](std::uintptr_t source, void* destination, std::size_t size) {
            if (Clock::now() >= deadline || size > validation_budget - validation_bytes) {
                if (!validation_limited) ++progress_.budget_limited_steps;
                validation_limited = true;
                return false;
            }
            validation_bytes += size;
            progress_.validation_bytes += size;
            return read(source, destination, size);
        };
        if (!progress_.scanning && now - completed_ms_ >= 5000) {
            cursor_ = 0;
            candidates_.clear();
            rejected_records_.clear();
            progress_.distinct_registries = progress_.distinct_catalogs = 0;
            unstable_ = false;
            progress_.scanning = true;
        }
        if (progress_.scanning) {
            if (buffer_.empty()) buffer_.resize(chunk_size + header_size);
            while (remaining >= header_size && Clock::now() < deadline) {
                MEMORY_BASIC_INFORMATION memory{};
                if (!(query ? query(cursor_, memory) :
                    VirtualQuery(reinterpret_cast<const void*>(cursor_), &memory, sizeof(memory)) == sizeof(memory))) {
                    Finish();
                    break;
                }
                const auto begin = reinterpret_cast<std::uintptr_t>(memory.BaseAddress);
                if (memory.RegionSize > UINTPTR_MAX - begin) {
                    Finish();
                    break;
                }
                const auto end = begin + memory.RegionSize;
                if (end <= cursor_) {
                    Finish();
                    break;
                }
                const auto protection = memory.Protect & 0xff;
                const bool readable = memory.State == MEM_COMMIT && memory.Type == MEM_PRIVATE &&
                    !(memory.Protect & (PAGE_GUARD | PAGE_NOACCESS)) &&
                    (protection == PAGE_READONLY || protection == PAGE_READWRITE ||
                     protection == PAGE_WRITECOPY || protection == PAGE_EXECUTE_READ ||
                     protection == PAGE_EXECUTE_READWRITE || protection == PAGE_EXECUTE_WRITECOPY);
                if (!readable || end - cursor_ < header_size) { cursor_ = end; continue; }
                const auto wanted = (std::min)({buffer_.size(), static_cast<std::size_t>(end - cursor_), remaining});
                const auto start = cursor_;
                remaining -= wanted;
                const auto chunk_end = start + wanted;
                const auto page_begin = start & ~(std::uintptr_t{page_size - 1});
                const auto page_count = static_cast<std::size_t>((chunk_end - 1 - page_begin) / page_size + 1);
                std::array<PSAPI_WORKING_SET_EX_INFORMATION, 66> pages{};
                if (page_count > pages.size()) {
                    Invalidate("resident-private-memory working-set query exceeds the page bound");
                    progress_.scanning = false;
                    completed_ms_ = now;
                    return false;
                }
                for (std::size_t i = 0; i < page_count; ++i)
                    pages[i].VirtualAddress = reinterpret_cast<void*>(page_begin + i * page_size);
                const bool queried = resident_query ? resident_query(pages.data(), page_count) :
                    K32QueryWorkingSetEx(GetCurrentProcess(), pages.data(),
                        static_cast<DWORD>(page_count * sizeof(pages[0]))) != FALSE;
                if (!queried) {
                    Invalidate("resident-private-memory working-set query failed");
                    progress_.scanning = false;
                    completed_ms_ = now;
                    return false;
                }
                // A working-set observation can race eviction. Never deliberately
                // fault in cold scan pages, and never bridge a nonresident gap.
                for (std::size_t page = 0; page < page_count;) {
                    const bool resident = pages[page].VirtualAttributes.Valid != 0;
                    const auto first = page++;
                    while (page < page_count && (pages[page].VirtualAttributes.Valid != 0) == resident) ++page;
                    const auto run_begin = (std::max)(start, page_begin + first * page_size);
                    const auto run_end = (std::min)(chunk_end, page_begin + page * page_size);
                    const auto run_size = static_cast<std::size_t>(run_end - run_begin);
                    if (!resident) {
                        progress_.skipped_nonresident_bytes += run_size;
                        cursor_ = run_end;
                        continue;
                    }
                    // Preserve a short trailing slice only if the next chunk
                    // can continue this resident run. No read spans a cold page.
                    const bool overlap = run_end == chunk_end && run_end < end;
                    if (run_size < header_size) {
                        cursor_ = overlap ? run_begin : run_end;
                        continue;
                    }
                    if (!read(run_begin, buffer_.data(), run_size)) {
                        // A racing page cannot hide the rest of the allocation.
                        cursor_ = (std::min)(run_end, (run_begin & ~(std::uintptr_t{page_size - 1})) + page_size);
                        break;
                    }
                    const auto buffer_begin = reinterpret_cast<std::uintptr_t>(buffer_.data());
                    std::size_t offset = (8 - (run_begin % 8)) % 8;
                    for (; offset + header_size <= run_size; offset += 8) {
                        const auto address = run_begin + offset;
                        // Bound header scanning inside each copied chunk too.
                        // Always advance a bounded batch before yielding, even
                        // when the preceding memory query consumed the budget.
                        if (inspected_headers && !(inspected_headers & 255) && Clock::now() >= deadline) {
                            cursor_ = address; // first unprocessed aligned header, with no progress counted yet
                            ++progress_.budget_limited_steps;
                            return false;
                        }
                        ++inspected_headers;
                        cursor_ = address + 8;
                        progress_.scanned_bytes += 8;
                        if (address >= buffer_begin && address - buffer_begin < buffer_.size()) continue;
                        if (!CandidateHeader(buffer_.data() + offset)) continue;
                        std::uintptr_t candidate_records{};
                        std::memcpy(&candidate_records, buffer_.data() + offset + 8, sizeof(candidate_records));
                        if (rejected_records_.contains(candidate_records)) continue;
                        Snapshot candidate;
                        std::string error;
                        // Check immutable image references before reading the
                        // complete registry and its parallel arrays.
                        const bool plausible = CandidateRecord(buffer_.data() + offset,
                            image, image_size, bounded);
                        if (!plausible && !validation_limited && rejected_records_.size() < 65536)
                            rejected_records_.insert(candidate_records);
                        if (plausible && ReadOwner(address, image, image_size, bounded, candidate, error) && validate(candidate, error))
                            RememberCandidate(std::move(candidate));
                        if (validation_limited) {
                            cursor_ = address;
                            progress_.scanned_bytes -= 8;
                            progress_.reason = "component registry candidate validation exceeded the step budget, retry pending";
                        }
                        if (plausible || validation_limited) return false;
                    }
                    // Overlap is confined to one contiguous resident run.
                    cursor_ = overlap ? run_begin + offset : run_end;
                }
            }
        }
        if (!selected_owner_ || (last_validation_ms_ && now - last_validation_ms_ < 1000)) return false;
        // Every known alias participates in publication. Equal metadata in two
        // different registries does not prove that either belongs to a world.
        std::unordered_set<std::uintptr_t> checked;
        Snapshot selected;
        const auto check = [&](const std::vector<Candidate>& candidates) {
            for (const auto& candidate : candidates) for (const auto owner : candidate.owners) {
                if (!checked.insert(owner).second) continue;
                Snapshot current;
                std::string error;
                if (!ReadOwner(owner, image, image_size, bounded, current, error) || !validate(current, error)) {
                    if (!validation_limited) Invalidate(error.empty() ? "known component registry owner no longer validates" : error);
                    return false;
                }
                if (selected_catalog_.entries != current.entries) {
                    Invalidate("known component registry owners expose different complete catalogs");
                    return false;
                }
                if (owner == selected_owner_) selected = std::move(current);
            }
            return true;
        };
        if (!check(known_candidates_) || !check(candidates_)) return false;
        if (selected.entries.empty()) {
            Invalidate("selected component registry owner is no longer known");
            return false;
        }
        output = std::move(selected);
        last_validation_ms_ = now;
        return true;
    }

private:
    static constexpr std::size_t header_size = 296, chunk_size = 256 * 1024, page_size = 4096;
    static constexpr std::size_t validation_budget = 4 * 1024 * 1024, max_candidates = 64;
    struct Candidate { Snapshot snapshot; std::vector<std::uintptr_t> owners; };
    Progress progress_;
    std::uintptr_t cursor_{}, selected_owner_{};
    std::uint64_t completed_ms_{};
    std::uint64_t last_validation_ms_{};
    bool overflow_{}, unstable_{};
    std::vector<unsigned char> buffer_;
    std::vector<Candidate> candidates_, known_candidates_;
    // Negative prefilter observations last for one scan only. Repeated small
    // heap values must not trigger thousands of identical remote/guarded reads.
    // A positive match always receives the complete fresh structural reader.
    std::unordered_set<std::uintptr_t> rejected_records_;
    Snapshot selected_catalog_;

    void Invalidate(const std::string& reason) {
        selected_owner_ = 0;
        progress_.available = false;
        progress_.reason = reason;
        last_validation_ms_ = 0;
    }

    void CountCatalogs() {
        progress_.distinct_registries = candidates_.size();
        progress_.distinct_catalogs = 0;
        for (std::size_t i = 0; i < candidates_.size(); ++i) {
            bool seen{};
            for (std::size_t j = 0; j < i; ++j)
                if (candidates_[i].snapshot.entries == candidates_[j].snapshot.entries) { seen = true; break; }
            if (!seen) ++progress_.distinct_catalogs;
        }
    }

    void RememberCandidate(Snapshot candidate) {
        // The first observation in a follow-up scan must also be compared with
        // the published catalog, not only with other candidates in this pass.
        if (selected_owner_ && selected_catalog_.entries != candidate.entries)
            Invalidate("new component registry candidate differs from the selected complete catalog");
        for (auto& previous : candidates_) if (previous.snapshot.records == candidate.records) {
            if (std::find(previous.owners.begin(), previous.owners.end(), candidate.owner) == previous.owners.end())
                previous.owners.push_back(candidate.owner);
            if (previous.snapshot.entries != candidate.entries) {
                unstable_ = true;
                Invalidate("component registry catalog changed during discovery");
            }
            return;
        }
        if (candidates_.size() >= max_candidates) {
            overflow_ = true;
            Invalidate("too many distinct component registry record arrays");
            return;
        }
        const auto owner = candidate.owner;
        candidates_.push_back({std::move(candidate), {owner}});
        CountCatalogs();
        if (progress_.distinct_catalogs > 1)
            Invalidate("multiple different complete component catalogs match the parser type contract");
    }

    static bool CandidateHeader(const unsigned char* bytes) {
        const auto word = [&](std::size_t offset) {
            std::uint64_t result{};
            std::memcpy(&result, bytes + offset, sizeof(result));
            return result;
        };
        const auto pointer = [](std::uint64_t value) {
            return value >= 0x10000 && value < 0x0000800000000000ULL && !(value & 7);
        };
        const auto count = word(16), capacity = word(24);
        return count >= 16 && count <= 1024 && capacity >= count && capacity <= 4096 &&
            word(240) == count && word(264) == count && word(288) == count &&
            pointer(word(8)) && pointer(word(232)) && pointer(word(256)) && pointer(word(280));
    }

    static bool CandidateRecord(const unsigned char* owner, std::uintptr_t image,
                                std::size_t image_size, const Reader& read) {
        std::uintptr_t records{};
        std::memcpy(&records, owner + 8, sizeof(records));
        unsigned char prefix[88]{};
        if (!read(records, prefix, sizeof(prefix))) return false;
        const auto word = [&](std::size_t offset) {
            std::uint64_t value{};
            std::memcpy(&value, prefix + offset, sizeof(value));
            return value;
        };
        const auto in_image = [&](std::uint64_t address, std::size_t size) {
            return address >= image && size <= image_size && address - image <= image_size - size;
        };
        const auto name = word(16), length = word(24), storage = word(40), configuration = word(48);
        if (length < 11 || length > 512 || !in_image(name, static_cast<std::size_t>(length)) ||
            (!storage && !configuration) || (storage && !in_image(storage, 0x90)) ||
            (configuration && !in_image(configuration, 0x90))) return false;
        char namespace_prefix[11]{};
        return read(name, namespace_prefix, sizeof(namespace_prefix)) &&
            std::memcmp(namespace_prefix, "keen::ecs::", sizeof(namespace_prefix)) == 0;
    }

    void Finish() {
        CountCatalogs();
        selected_owner_ = !overflow_ && !unstable_ && progress_.distinct_catalogs == 1 ?
            candidates_.front().snapshot.owner : 0;
        if (selected_owner_) {
            selected_catalog_ = candidates_.front().snapshot;
            known_candidates_ = candidates_;
        }
        progress_.available = selected_owner_ != 0;
        last_validation_ms_ = 0;
        progress_.scanning = false;
        ++progress_.completed_passes;
        completed_ms_ = GetTickCount64();
        progress_.reason = overflow_ ? "too many distinct component registry record arrays" :
            unstable_ ? "component registry catalog changed during discovery" :
            candidates_.empty() ? "no component registry in resident-private-memory matches the parser type contract" :
            progress_.distinct_catalogs != 1 ? "multiple different complete component catalogs found in resident-private-memory" :
            "validated equivalent catalogs in resident-private-memory, entity access requires a world manager and dispatcher";
        overflow_ = false;
    }
};
}
