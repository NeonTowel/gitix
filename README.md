# Gitix

A beautiful Git TUI (Terminal User Interface) built with Rust and Ratatui, featuring the soothing [Catppuccin](https://catppuccin.com/) color palette.

## Features

- 🎨 **Beautiful Catppuccin Theme** - Soothing pastel colors with configurable accent colors
- 📊 **Repository Overview** - Commit statistics, activity calendar, and recent changes
- 📁 **File Browser** - Navigate and open files with your preferred editor
- 📋 **Git Status** - View modified, staged, and untracked files
- 💾 **Save Changes** - Stage files and create commits with ease
- 🔄 **Update Repository** - Pull latest changes (coming soon)
- ⚙️ **Settings** - Configure your Git TUI experience

## 🎨 Catppuccin Theme System

Gitix uses the beautiful [Catppuccin color palette](https://catppuccin.com/palette/) with the **Macchiato** flavor, following the official color usage guidelines:

### Color Semantics

Following [Catppuccin's style guide](https://catppuccin.com/palette/):

- **Base** (`#24273a`) - Main terminal/UI background
- **Mantle** (`#1e2030`) - Secondary backgrounds (panels, tabs)
- **Crust** (`#181926`) - Deepest backgrounds (status bars, borders)
- **Text** (`#cad3f5`) - Primary text content
- **Accent Colors** - Interactive elements (active tabs, focus indicators, highlights)

### Configurable Accent Colors

The theme supports **14 different accent colors** that you can choose from:

```rust
use gitix::{Theme, AccentColor};

// Using convenience constructors
let theme = Theme::pink();      // 🌸 Pink accent
let theme = Theme::lavender();  // 💜 Lavender accent
let theme = Theme::green();     // 🌿 Green accent
let theme = Theme::blue();      // 💙 Blue accent (default)

// Using the with_accent method
let theme = Theme::with_accent(AccentColor::Blue);  // Default
let theme = Theme::with_accent(AccentColor::Peach);

// Change accent color at runtime
let mut theme = Theme::new();  // Starts with blue accent
theme.set_accent(AccentColor::Teal);
```

### Available Accent Colors

| Color        | Hex       | Usage Example             |
| ------------ | --------- | ------------------------- |
| 🌹 Rosewater | `#f4dbd6` | `Theme::rosewater()`      |
| 🦩 Flamingo  | `#f0c6c6` | `Theme::flamingo()`       |
| 🌸 Pink      | `#f5bde6` | `Theme::pink()`           |
| 🔮 Mauve     | `#c6a0f6` | `Theme::mauve()`          |
| ❤️ Red       | `#ed8796` | `Theme::red()`            |
| 🍷 Maroon    | `#ee99a0` | `Theme::maroon()`         |
| 🍑 Peach     | `#f5a97f` | `Theme::peach()`          |
| 💛 Yellow    | `#eed49f` | `Theme::yellow()`         |
| 🌿 Green     | `#a6da95` | `Theme::green()`          |
| 🌊 Teal      | `#8bd5ca` | `Theme::teal()`           |
| ☁️ Sky       | `#91d7e3` | `Theme::sky()`            |
| 💎 Sapphire  | `#7dc4e4` | `Theme::sapphire()`       |
| 💙 Blue      | `#8aadf4` | `Theme::blue()` (default) |
| 💜 Lavender  | `#b7bdf8` | `Theme::lavender()`       |

### Theme Demo

Run the theme demonstration to see all available colors:

```bash
cargo run --example theme_demo
```

## Installation

```bash
git clone https://github.com/yourusername/gitix.git
cd gitix
cargo build --release
```

## Usage

Navigate to any directory and run:

```bash
cargo run
```

### Keyboard Shortcuts

- **Tab** / **Shift+Tab** - Navigate between tabs
- **↑↓** - Navigate within lists
- **Enter** - Open files or confirm actions
- **Space** - Stage/unstage files (in Save Changes tab)
- **q** - Quit application

## Development

### Project Structure

```
src/
├── app.rs          # Application state management
├── files.rs        # File system operations
├── git.rs          # Git operations
├── main.rs         # Entry point
├── lib.rs          # Library exports
└── tui/            # Terminal UI components
    ├── mod.rs      # Main TUI loop
    ├── theme.rs    # Catppuccin theme system
    ├── overview.rs # Repository overview tab
    ├── files.rs    # File browser tab
    ├── status.rs   # Git status tab
    ├── save_changes.rs # Commit interface
    ├── update.rs   # Update repository tab
    └── settings.rs # Settings tab
```

### Dependencies

- **ratatui** - Terminal UI framework
- **crossterm** - Cross-platform terminal manipulation
- **gix** - Pure Rust Git implementation
- **catppuccin** - Official Catppuccin color palette
- **chrono** - Date and time handling
- **tui-textarea** - Text input widget

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Catppuccin](https://catppuccin.com/) for the beautiful color palette
- [Ratatui](https://ratatui.rs/) for the excellent TUI framework
- [gix](https://github.com/Byron/gitoxide) for Git operations
