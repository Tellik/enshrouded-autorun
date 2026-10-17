#include "profile_cli.h"

namespace profile_development {
int inspect_command(int argc, char** argv) {
    const fs::path exe = argv[2];
    std::string error;
    auto pe = read_pe(exe, error);
    if (!pe) { std::cerr << error << '\n'; return 2; }
    const auto report = image_report(exe, *pe);
    if (const auto out = option(argc, argv, "--out")) {
        if (!write_json(*out, report)) return 2;
        std::cout << "image report: " << fs::absolute(*out).string() << '\n';
    } else std::cout << report.dump(2) << '\n';
    return 0;
}
}
