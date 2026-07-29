## Rules

### Vendoring External Repositories
When vendoring or cloning external open-source repositories into the project workspace:
1. **Aggressively Clean Boilerplate**: Immediately delete files that belong to the original repository's open-source scaffolding rather than the compiled project. This includes `CODE_OF_CONDUCT.md`, `CONTRIBUTING.md`, `LICENSE*`, `SECURITY.md`, `README.md`, and any `.git` folders.
2. **Verify Mathematical Authenticity**: Do not blindly trust complex control algorithms, physics engines, or cryptographic math found in cloned repositories. Always identify the underlying academic paper or theorem (e.g., Minimum Snap Trajectories via Mellinger & Kumar) and perform a sanity check to verify the code accurately reflects the paper's equations.
