#include "profile_cli.h"
using namespace profile_development;

namespace profile_development {
void usage() {
    std::cout <<
        "kfc-runtime-dev - build-time Enshrouded runtime investigation tools\n\n"
        "Commands:\n"
        "  select-build <game-directory-or-exe> [--out-dir <capture-directory>] [--catalog-directory <catalog-directory>]\n"
        "  inspect <enshrouded.exe> [--out <image-report.json>]\n"
        "  discover-functions <enshrouded.exe> [--out <function-candidates.json>]\n"
        "  extract-profile-functions <profile.json> [--out <function-catalog.json>]\n"
        "  scan-functions <enshrouded.exe> <function-catalog.json> [--out <scan-report.json>]\n"
        "  capture-process <pid> [--out <process-capture.json>]\n"
        "  import-ecs-capture <capture.log> --image-report <image.json> --reflection <kfc-parser-reflection.json> [--id <profile-id>] [--out <component-catalog.json>]\n"
        "  inspect-provider <kfc-runtime.dll> [--out <provider-report.json>]\n"
        "  validate-profile <profile.json>\n"
        "  approve-profile <draft.json> --function-scan <scan.json> --components <catalog.json>\n"
        "  generate-profile <base-profile.json> <image-report.json> <scan-report.json> --kfc-format <3|4> --components <live-capture.json> --out <draft.json>\n\n"
        "This console is developer-only. It is not run by the game or modloader.\n";
}
}

int main(int argc, char** argv) {
    if (argc < 2 || std::string_view(argv[1]) == "--help" || std::string_view(argv[1]) == "help") { usage(); return argc < 2 ? 1 : 0; }
    const std::string_view command(argv[1]);
    if (command == "select-build" && argc >= 3) return select_build_command(argc, argv);
    if (command == "inspect" && argc >= 3) return inspect_command(argc, argv);
    if (command == "discover-functions" && argc >= 3) return discover_command(argc, argv);
    if (command == "extract-profile-functions" && argc >= 3) return extract_profile_functions_command(argc, argv);
    if (command == "scan-functions" && argc >= 4) return scan_command(argc, argv);
    if (command == "capture-process" && argc >= 3) return capture_process_command(argc, argv);
    if (command == "import-ecs-capture" && argc >= 3) return import_ecs_capture_command(argc, argv);
    if (command == "inspect-provider" && argc >= 3) return inspect_provider_command(argc, argv);
    if (command == "validate-profile" && argc >= 3) return validate_command(argv[2], option(argc, argv, "--components"));
    if (command == "approve-profile" && argc >= 3) return approve_profile_command(argc, argv);
    if (command == "generate-profile" && argc >= 5) return generate_command(argc, argv);
    usage();
    return 1;
}
