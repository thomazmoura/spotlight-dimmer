# Agent Instructions for SpotlightDimmer

This file contains instructions for AI agents (Claude Code) working on this repository.

## Changelog Management (REQUIRED)

**CRITICAL**: Every code change, feature addition, bug fix, or improvement MUST be documented in `CHANGELOG.md`. This is not optional.

### When to Update the Changelog

Update `CHANGELOG.md` for ANY of these changes:
- ✅ New features or functionality
- ✅ Bug fixes and issue resolutions
- ✅ Breaking changes or API modifications
- ✅ Performance improvements
- ✅ UI/UX enhancements
- ✅ Configuration changes
- ✅ Dependency updates (if user-facing)
- ✅ Security fixes
- ✅ Documentation improvements (if significant)

### Changelog Format

Follow [Keep a Changelog](https://keepachangelog.com/) format. Always add entries under the `## [Unreleased]` section:

#### For New Features
```markdown
### Added
- Feature name: Clear description of what it does and why it's useful for users
- Another feature: Focus on user-facing benefits, not internal implementation details
```

#### For Bug Fixes
```markdown
### Fixed
- Issue description: What was broken and how it affects users
- Bug name: Clear explanation of the fix and its impact
```

#### For Breaking Changes
```markdown
### Changed
- Breaking change description: What changed and why
- Migration steps: If users need to take action, explain how
```

#### For Performance/Internal Improvements
```markdown
### Improved
- Performance enhancement: Measurable impact on user experience
- Internal optimization: Only if it affects user-visible behavior
```

#### For Removed Features
```markdown
### Removed
- Deprecated feature: What was removed and why
- Alternative solution: What users should use instead
```

### Portuguese Translation Requirement (MANDATORY)

**CRITICAL**: Every changelog entry MUST include a Portuguese translation. This is required for all entries without exception.

#### Format Structure
Each changelog entry must follow this bilingual format:
```markdown
### Added
- Feature name: Clear description of what it does and why it's useful for users
- Another feature: Focus on user-facing benefits, not internal implementation details

---

### Adicionado
- Nome da funcionalidade: Descrição clara do que faz e por que é útil para os usuários
- Outra funcionalidade: Foque nos benefícios voltados ao usuário, não em detalhes de implementação interna
```

#### Translation Guidelines
1. **Maintain Technical Accuracy**: Ensure technical terms are correctly translated or kept in English when appropriate
2. **User-Friendly Language**: Use Portuguese that Brazilian and Portuguese users can easily understand
3. **Consistent Terminology**: Keep consistent translations for recurring technical terms
4. **Section Headers**: Always translate section headers (Added→Adicionado, Fixed→Corrigido, Changed→Alterado, etc.)

#### Section Header Translations
- **Added** → **Adicionado**
- **Fixed** → **Corrigido**
- **Changed** → **Alterado**
- **Improved** → **Melhorado**
- **Removed** → **Removido**
- **Security** → **Segurança**
- **Deprecated** → **Obsoleto**

### Changelog Writing Guidelines

1. **User-Focused**: Write for end users, not developers
2. **Clear Impact**: Explain what changed and why it matters
3. **Actionable**: Include migration steps for breaking changes
4. **Specific**: Use concrete examples rather than vague descriptions
5. **Consistent**: Follow the same style and format for all entries
6. **Bilingual**: Always include Portuguese translations using the format above

### Example Entry
```markdown
### Added
- Dark mode support: Users can now switch between light and dark themes via configuration
- Keyboard shortcuts: Added Ctrl+D to toggle dimming and Ctrl+Q to quit application
- Multi-monitor performance: Reduced CPU usage by 40% when managing 3+ displays through event-driven architecture

### Fixed
- Display detection bug: Application now properly detects displays after sleep/wake cycles
- Memory leak: Fixed DeferWindowPos handle leak causing memory growth during window dragging

### Changed
- Overlay transparency: Changed default dimming from 30% to 60% for better visibility (users can adjust in config.json)

---

### Adicionado
- Suporte a modo escuro: Os usuários agora podem alternar entre temas claro e escuro através da configuração
- Atalhos de teclado: Adicionado Ctrl+D para alternar o escurecimento e Ctrl+Q para sair da aplicação
- Performance multi-monitor: Redução de 40% no uso de CPU ao gerenciar 3+ displays através de arquitetura orientada a eventos

### Corrigido
- Bug de detecção de display: A aplicação agora detecta adequadamente displays após ciclos de suspensão/despertar
- Vazamento de memória: Corrigido vazamento de handle DeferWindowPos causando crescimento de memória durante arrasto de janelas

### Alterado
- Transparência de sobreposição: Alterada transparência padrão de 30% para 60% para melhor visibilidade (usuários podem ajustar em config.json)
```

### Release Process Integration

The changelog directly feeds into GitHub Releases:
- Release notes are automatically generated from the `[Unreleased]` section
- This ensures comprehensive, professional release documentation
- No manual release note writing required

### Quality Standards

Each changelog entry should answer:
- **What** changed?
- **Why** did it change?
- **How** does it affect users?
- **What** should users do (if action required)?
- **Is the Portuguese translation accurate and user-friendly?**

**Enforcement**: Changes without proper changelog updates (including Portuguese translations) will be considered incomplete.

## Version Management

Version management is centralized in `Directory.Build.props` which is automatically imported by all .NET projects in the repository.

### Version File Location
- **File**: `Directory.Build.props`
- **Properties to update**:
  - `<Version>` - Full version including pre-release suffix (e.g., 0.8.0-beta)
  - `<AssemblyVersion>` - Major.Minor.Patch only (e.g., 0.8.0)
  - `<FileVersion>` - Major.Minor.Patch only (e.g., 0.8.0)
  - `<InformationalVersion>` - Full version including pre-release suffix (e.g., 0.8.0-beta)

### Automated Release Commands

Use the slash commands for version bumps and releases:

- `/publish-patch` - Increment patch version (0.8.0 → 0.8.1) for bug fixes
- `/publish-minor` - Increment minor version (0.8.1 → 0.9.0) for new features

These commands:
1. Update version in `Directory.Build.props`
2. Run validation (build + tests)
3. Create git commit and tag
4. Push to repository

See `.claude/commands/publish-patch.md` and `.claude/commands/publish-minor.md` for detailed documentation.

## Git Commit Messages

**DO NOT** include "Generated with Claude Code" or "Co-Authored-By: Claude" in commit messages. This repository is built with Claude Code - these attributions are redundant.

Use clear, descriptive commit messages:
- Subject line (50 chars max)
- Brief explanation of what changed and why
- List specific changes if multiple

### Example Commit Message Format:
```
Fix DeferWindowPos handle leak during window dragging

Implemented proper handle cleanup to prevent memory growth during drag operations.

- Clean up last valid handle on DeferWindowPos failure
- Add verbose GDI object monitoring
- Document handle leak prevention pattern
```

## Architecture Guidelines

### Layer Separation
- **Core layer** (`Core/` directory): Pure C# logic with zero Windows dependencies
- **WindowsBindings layer** (`WindowsBindings/` directory): Windows-specific P/Invoke and Win32 API integration
- **Main program** (`Program.cs`): Wires Core and WindowsBindings together

**CRITICAL**: Never add Windows dependencies to the Core layer. Keep it pure and platform-agnostic.

### Zero-Allocation Hot Path
The codebase is designed to eliminate allocations during window movement and focus changes:

1. **Pre-allocation**: All overlays created at startup
2. **In-place updates**: Use `CopyFrom()` methods instead of creating new objects
3. **Cached values**: Cache configuration and display info to avoid re-allocation
4. **Batch operations**: Use `DeferWindowPos` for atomic window updates

**When modifying update logic**:
- Never allocate in event handlers (`FocusedDisplayChanged`, `WindowPositionChanged`)
- Use `CopyFrom()` pattern for updating existing objects
- Reuse pre-allocated collections (e.g., `_updateBatch` in OverlayRenderer)

### Memory Leak Prevention

**DeferWindowPos Handle Management**:
- Always call `EndDeferWindowPos()` even on failure
- Track `lastValidHdwp` to ensure cleanup of correct handle
- See `OverlayRenderer.cs:105-153` for the pattern

**GDI Object Monitoring**:
- Use `--verbose` flag to enable GDI object count logging
- Helps detect brush/window handle leaks during development
- See `Program.cs:136-179` for implementation

## Development Best Practices

### Code Style
- Use XML doc comments (`///`) for all public APIs
- Prefer `ReadOnlySpan<T>` for array parameters
- Use structs for hot-path data (Color, Rectangle, Point)
- Keep Core layer pure - no Windows dependencies

### Testing
- Test programs are in `Test*.cs` files (not NUnit/xUnit)
- To run: Uncomment the test's `Run()` call in Program.cs
- Add new tests following the same pattern

### Performance
- Measure GDI object counts with `--verbose` flag
- Profile hot paths to ensure zero allocations
- Use event-driven architecture - never add polling loops

## Common Development Scenarios

### Adding a New Dimming Mode
1. Add enum value to `Core/DimmingMode.cs`
2. Add case to `AppState.Calculate()` switch statement
3. Implement update logic method (follow pattern of `UpdatePartialOverlays()`)
4. Update `CONFIGURATION.md` with new mode documentation
5. **Update CHANGELOG.md** with the new feature (including Portuguese translation)

### Modifying Overlay Rendering
1. Core logic changes go in `Core/AppState.cs`
2. Windows-specific changes go in `WindowsBindings/OverlayRenderer.cs`
3. Maintain separation: Core has no Windows dependencies
4. Ensure zero allocations in update paths
5. **Update CHANGELOG.md** with improvements (including Portuguese translation)

### Adding Configuration Options
1. Add property to `Core/AppConfig.cs`
2. Update `ToOverlayConfig()` method to map to `OverlayCalculationConfig`
3. Handle in `Program.cs` ConfigurationChanged event if needed
4. Document in `CONFIGURATION.md`
5. **Update CHANGELOG.md** with configuration changes (including Portuguese translation)

## Quality Checklist

Before completing any task, verify:
- [ ] Code follows layer separation (Core vs WindowsBindings)
- [ ] No allocations in hot path (event handlers, Calculate, UpdateOverlays)
- [ ] Memory leaks prevented (handles properly cleaned up)
- [ ] XML doc comments added for public APIs
- [ ] Configuration documented in CONFIGURATION.md if applicable
- [ ] **CHANGELOG.md updated with changes**
- [ ] **Portuguese translation included in CHANGELOG.md**
- [ ] Commit message is clear and descriptive
- [ ] No "Generated with Claude Code" in commit messages
