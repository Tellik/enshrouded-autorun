#pragma once
#include <cstdint>
#include <cstring>
#include <vector>

namespace WandDurability {
struct HookCode {
    std::vector<std::uint8_t> bytes;
    std::size_t charge_jump{}, skip_jump{};
};

inline HookCode make_hook(std::uintptr_t callback) {
    // At RVA 0x225450, RSP is 16-byte aligned. Save flags and all volatile
    // GPRs (64 bytes), then 32 bytes of shadow space + six XMM registers.
    HookCode result;
    auto& code = result.bytes;
    code = {0x9c,0x50,0x51,0x52,0x41,0x50,0x41,0x51,0x41,0x52,0x41,0x53,
        0x48,0x81,0xec,0x80,0,0,0};
    for (unsigned i = 0; i < 6; ++i)
        code.insert(code.end(), {0xf3,0x0f,0x7f,static_cast<std::uint8_t>(0x44+i*8),
            0x24,static_cast<std::uint8_t>(0x20+i*16)});
    // RCX = original Actor* from this system's binding frame. RDX = the
    // Durability* already resolved by the original tracked mutable accessor.
    code.insert(code.end(), {0xfc,0x48,0x8b,0x8d,0x90,0x03,0,0,0x48,0x89,0xda,
        0x44,0x8b,0x00,0x48,0xb8}); // R8D = original system actor entity at [RAX]
    for (unsigned i = 0; i < 8; ++i) code.push_back(static_cast<std::uint8_t>(callback >> (8*i)));
    code.insert(code.end(), {0xff,0xd0,0x84,0xc0,0x0f,0x85,0,0,0,0});
    const auto branch_end = code.size();
    auto restore = [&] {
        for (unsigned i = 0; i < 6; ++i)
            code.insert(code.end(), {0xf3,0x0f,0x6f,static_cast<std::uint8_t>(0x44+i*8),
                0x24,static_cast<std::uint8_t>(0x20+i*16)});
        code.insert(code.end(), {0x48,0x81,0xc4,0x80,0,0,0,0x41,0x5b,0x41,0x5a,0x41,0x59,
            0x41,0x58,0x5a,0x59,0x58,0x9d});
    };
    restore();
    // Replay only the displaced instruction. The game performs its original
    // tracked durability subtraction and sends its normal notifications.
    code.insert(code.end(), {0x41,0xb8,0xff,0xff,0xff,0xff});
    result.charge_jump = code.size();
    code.insert(code.end(), {0xe9,0,0,0,0});
    const auto displacement = static_cast<std::int32_t>(code.size() - branch_end);
    std::memcpy(code.data() + branch_end - 4, &displacement, 4);
    restore();
    result.skip_jump = code.size();
    code.insert(code.end(), {0xe9,0,0,0,0});
    return result;
}
}
