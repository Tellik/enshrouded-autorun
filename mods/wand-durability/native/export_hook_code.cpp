#include "durability_hook_code.h"
#include <fstream>
#include <iostream>
int main(int argc, char** argv) {
    if (argc != 2) return 1;
    const auto code = WandDurability::make_hook(0x300000);
    std::ofstream output(argv[1], std::ios::binary);
    output.write(reinterpret_cast<const char*>(code.bytes.data()), code.bytes.size());
    std::cout << code.charge_jump << ' ' << code.skip_jump << '\n';
    return output ? 0 : 1;
}
