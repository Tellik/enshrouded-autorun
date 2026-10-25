#include "input_observation.h"

#include <cstdlib>
#include <iostream>

namespace {
void require(bool condition, const char* message) {
    if (!condition) {
        std::cerr << message << '\n';
        std::exit(1);
    }
}

void qualify(AutorunInput::Observation& observation, const void* input,
             std::uint64_t session, std::uint64_t start,
             std::uint64_t duration, std::uint64_t hits) {
    for (std::uint64_t i = 0; i < hits; ++i)
        observation.observe(input, session, start + i * duration / (hits - 1));
}
}

int main() {
    using namespace AutorunInput;
    int original{}, replacement{};
    Observation observation;

    qualify(observation, &original, 1, 1000, InitialObservationMs, InitialObservationHits);
    require(observation.snapshot(1, 1500).stable_input == &original,
            "initial input did not qualify");
    require(observation.observe(&replacement, 1, 1510), "replacement was not detected");
    require(observation.snapshot(1, 1510).readiness == Readiness::Revalidating,
            "replacement must immediately disable movement");
    qualify(observation, &replacement, 1, 1520,
            ReplacementObservationMs - 20, ReplacementObservationHits - 2);
    require(observation.snapshot(1, 3000).readiness == Readiness::Revalidating,
            "replacement qualified too early");
    observation.observe(&replacement, 1, 3010);
    require(observation.snapshot(1, 3010).stable_input == &replacement,
            "stable replacement did not recover in the same session");
    require(observation.snapshot(1, 3010).replacements == 1,
            "replacement count was lost");

    for (std::uint64_t i = 0; i < 100; ++i) {
        observation.observe(i % 2 ? &replacement : &original, 1, 3020 + i * 20);
        require(observation.snapshot(1, 3020 + i * 20).stable_input == nullptr,
                "alternating inputs must not qualify");
    }

    qualify(observation, &original, 1, 6000, ReplacementObservationMs,
            ReplacementObservationHits);
    require(observation.snapshot(1, 7500).stable_input == &original,
            "stable input did not recover after alternation");
    require(observation.snapshot(1, 8501).readiness == Readiness::Stale,
            "old input must expire without callbacks");
    observation.observe(&original, 1, 8502);
    require(observation.snapshot(1, 8502).stable_input == nullptr,
            "old samples must not survive an idle gap");

    require(observation.snapshot(2, 8502).readiness == Readiness::DifferentSession,
            "old session must not qualify in a new world");
    qualify(observation, &original, 2, 9000, InitialObservationMs, InitialObservationHits);
    require(observation.snapshot(2, 9500).stable_input == &original,
            "new world should use initial qualification");
    require(observation.snapshot(2, 9500).replacements == 0,
            "replacement count must reset in a new world");
}
