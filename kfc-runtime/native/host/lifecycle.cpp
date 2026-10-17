#include "host/provider_state.h"



using namespace ProviderImplementation;

namespace ProviderImplementation {
std::atomic<bool> provider_initialized{};
}

extern "C" {
KFC_EXPORT unsigned __cdecl KfcRuntimeAbi() { return KFC_RUNTIME_ABI_VERSION; }

KFC_EXPORT bool __cdecl KfcRuntimeInitialize() {
    try {
        const auto initialized = EcsRuntime::Initialize();
        ResourceRuntime::Initialize();
        provider_initialized.store(true, std::memory_order_release);
        return initialized;
    } catch (...) {
        provider_initialized.store(false, std::memory_order_release);
        ResourceRuntime::Shutdown();
        EcsRuntime::Shutdown();
        return false;
    }
}

KFC_EXPORT void __cdecl KfcRuntimeTick() {
    try { EcsRuntime::Tick(); } catch (...) { EcsRuntime::Shutdown(); }
}

KFC_EXPORT void __cdecl KfcRuntimeShutdown() {
    provider_initialized.store(false, std::memory_order_release);
    KfcRuntimeNetworkServiceShutdown();
    ResourceRuntime::Shutdown();
    EcsRuntime::Shutdown();
}
}
