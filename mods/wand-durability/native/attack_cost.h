#pragma once
#include <array>
#include <cstddef>
#include <cstdint>

namespace WandDurability {
// ItemInfo.sequences[].sequenceId for all nine primary wand attacks in
// build 26082781. Derived from the original resources, not display names.
constexpr std::array<std::uint32_t, 9> WandSequences{
    3820206067u, 2862649328u, 2411228084u, 2611236789u, 422768056u,
    2918815211u, 1422481419u, 1312513725u, 2630637865u};

constexpr bool is_wand_sequence(std::uint32_t id) {
    for (auto candidate : WandSequences) if (candidate == id) return true;
    return false;
}

struct Attack {
    std::uint64_t session{};
    std::uintptr_t actor{}, durability{};
    std::uint32_t sequence{}, actor_entity{}, weapon_entity{};
    std::uint64_t trigger_time{};
    std::uint8_t call_index{};
    bool operator==(const Attack&) const = default;
};

// Called only at the original successful-projectile durability call, after
// the world's enableDurability check. No clocks or projectile-count guesses.
// A bounded cache never retains engine pointers for later dereferencing.
class AttackCosts {
    std::array<Attack, 128> attacks_{};
    std::size_t next_{};
public:
    bool suppress(const Attack& attack) {
        if (!attack.session || !attack.actor || !attack.durability ||
            !attack.actor_entity || !attack.weapon_entity ||
            !is_wand_sequence(attack.sequence)) return false;
        // A shared world can interleave several players' projectile events.
        // Keep identities independently, even when their entity IDs hash to
        // the same slot. Saturation only forgets old attacks, so it can never
        // grant a free durability charge to an unknown attack.
        for (const auto& previous : attacks_)
            if (previous == attack) return true;
        attacks_[next_] = attack;
        next_ = (next_ + 1) % attacks_.size();
        return false;
    }
};
}
