#include "attack_cost.h"
#include <cstdlib>
#include <iostream>

void check(bool value, const char* message) {
    if (!value) { std::cerr << message << '\n'; std::exit(1); }
}

int main() {
    using namespace WandDurability;
    for (auto sequence : WandSequences) {
        AttackCosts costs;
        Attack a{1, 0x1000, 0x2000, sequence, 42, 73, 100, 0};
        check(!costs.suppress(a), "first successful projectile costs one");
        for (int n = 0; n < 32; ++n)
            check(costs.suppress(a), "all bonus projectiles of one attack cost zero");
        a.trigger_time++; // even immediately adjacent attacks are distinct
        check(!costs.suppress(a), "next attack still costs one");
        a.call_index++;
        check(!costs.suppress(a), "same tick, different action costs one");
        a.weapon_entity++;
        check(!costs.suppress(a), "another weapon cannot inherit a free charge");
        a.session++;
        check(!costs.suppress(a), "world rejoin invalidates previous charge");
        a.actor += 16;
        check(!costs.suppress(a), "replacement actor cannot inherit a free charge");
        a.durability += 16;
        check(!costs.suppress(a), "replacement item component costs one");
        auto b = a; b.actor_entity++; b.actor += 16;
        check(!costs.suppress(b), "another player's first projectile costs one");
        check(costs.suppress(a) && costs.suppress(b), "players are independent");
        b.actor_entity = a.actor_entity + 128;
        check(!costs.suppress(b), "same hash slot still costs on first projectile");
        check(costs.suppress(a) && costs.suppress(b), "hash collision cannot lose either attack");
        AttackCosts bounded;
        check(!bounded.suppress(a), "first entry is charged");
        for (std::uint32_t n = 0; n < 128; ++n) {
            auto other = a;
            other.actor_entity = 1000 + n;
            other.actor += (n + 1) * 16;
            check(!bounded.suppress(other), "distinct attacks are charged");
        }
        check(!bounded.suppress(a), "journal saturation preserves original cost");
        a.sequence = 0;
        check(!costs.suppress(a) && !costs.suppress(a), "non-wand costs never change");
        a.sequence = sequence; a.session = 0;
        check(!costs.suppress(a), "unknown world preserves original cost");
        a.session = 1; a.weapon_entity = 0;
        check(!costs.suppress(a), "unknown weapon preserves original cost");
    }
    std::cout << "Attack cost cases passed for all nine wand sequences\n";
}
