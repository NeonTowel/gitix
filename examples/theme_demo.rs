use gitix::tui::theme::{AccentColor, Theme, TitleColor};

fn main() {
    println!("🎨 Catppuccin Theme Demo");
    println!("========================");
    println!();

    // Example 1: Using default theme (blue accent)
    let default_theme = Theme::new();
    println!("✨ Default theme uses blue accent (updated!)");

    // Example 2: Using convenience constructors
    let pink_theme = Theme::pink();
    let lavender_theme = Theme::lavender();
    let green_theme = Theme::green();

    println!("🌸 Pink theme created with Theme::pink()");
    println!("💜 Lavender theme created with Theme::lavender()");
    println!("🌿 Green theme created with Theme::green()");
    println!();

    // Example 3: Using with_accent method
    let custom_theme = Theme::with_accent(AccentColor::Blue);
    println!("🔮 Custom theme with blue accent");
    println!();

    // Example 4: Changing accent color at runtime
    let mut runtime_theme = Theme::new();
    println!("🔄 Runtime theme changes:");
    println!("  - Started with: {:?} (new default!)", AccentColor::Blue);

    runtime_theme.set_accent(AccentColor::Peach);
    println!("  - Changed to: {:?}", AccentColor::Peach);

    runtime_theme.set_accent(AccentColor::Teal);
    println!("  - Changed to: {:?}", AccentColor::Teal);
    println!();

    // Example 5: NEW - Default accent colors
    println!("🎯 New default accent colors:");
    println!("  - Primary (accent): Blue - used for tabs, focus, active elements");
    println!("  - Secondary (accent2): Rosewater - used for labels, authors");
    println!("  - Tertiary (accent3): Mauve - used for timestamps, metadata");
    println!();

    // Example 6: NEW - Title color configuration
    println!("📋 Title color configuration:");
    let overlay0_theme = Theme::with_overlay0_titles();
    println!("  - Default: overlay0 titles (subtle, recommended)");

    let overlay1_theme = Theme::with_overlay1_titles();
    println!("  - overlay1 titles (slightly more prominent)");

    let text_theme = Theme::with_text_titles();
    println!("  - text color titles (high contrast)");

    let accent_title_theme = Theme::with_accent_titles(AccentColor::Pink);
    println!("  - accent color titles (pink accent for headers)");

    // Runtime title color changes
    let mut title_theme = Theme::new();
    println!("  - Runtime changes:");
    title_theme.set_title_color(TitleColor::Overlay1);
    println!("    → Changed to overlay1");
    title_theme.set_title_color(TitleColor::Accent(AccentColor::Pink));
    println!("    → Changed to pink accent");
    println!();

    // Example 7: All available accent colors
    println!("🎨 Available accent colors:");
    let accent_colors = [
        AccentColor::Rosewater,
        AccentColor::Flamingo,
        AccentColor::Pink,
        AccentColor::Mauve,
        AccentColor::Red,
        AccentColor::Maroon,
        AccentColor::Peach,
        AccentColor::Yellow,
        AccentColor::Green,
        AccentColor::Teal,
        AccentColor::Sky,
        AccentColor::Sapphire,
        AccentColor::Blue,
        AccentColor::Lavender,
    ];

    for color in accent_colors {
        let theme = Theme::with_accent(color);
        println!(
            "  - {:?}: Use Theme::with_accent(AccentColor::{:?})",
            color, color
        );
    }
    println!();

    println!("💡 Usage in your TUI:");
    println!("   let theme = Theme::new();  // Now uses blue accent by default!");
    println!("   let theme = Theme::pink();  // For pink accent");
    println!("   let theme = Theme::with_accent(AccentColor::Lavender);  // For lavender accent");
    println!("   theme.set_accent(AccentColor::Green);  // Change at runtime");
    println!("   theme.set_title_color(TitleColor::Overlay0);  // Set title color");
    println!();

    println!("🎯 Semantic color usage (following Catppuccin guidelines):");
    println!("   - Base: Tab bar and status bar backgrounds");
    println!("   - Mantle: Unified background for terminal and all content");
    println!("   - Crust: Deepest accent color");
    println!("   - Blue (Primary): Navigation tabs, focus indicators, active elements");
    println!("   - Rosewater (Secondary): Labels, authors, secondary highlights");
    println!("   - Mauve (Tertiary): Timestamps, metadata");
    println!("   - Overlay0: Panel headers/titles (default, subtle)");
    println!("   - Text: Always use for text content");
}
