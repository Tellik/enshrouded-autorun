#include "profile_commands.h"

namespace profile_development {
std::optional<fs::path> option(int argc, char** argv, std::string_view key) {
    for (int i = 0; i + 1 < argc; ++i) if (std::string_view(argv[i]) == key) return fs::path(argv[i + 1]);
    return std::nullopt;
}
}
