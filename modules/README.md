# Product modules

Each direct child of `modules/` is a product module with its own behavior and, where needed, its own `ui/` directory. `modules/transport/` is a module too. A window passes work to its owning module and presents the result instead of implementing the same behavior again.

See [code ownership](../docs/development/code-ownership.en.md) for the current repository map and action-to-file guide.
