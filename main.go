package main

import (
	"fmt"

	"github.com/gdamore/tcell/v2"
	"github.com/neontowel/gitix/save"
	"github.com/rivo/tview"
)

// FocusZone represents a UI focus zone
type FocusZone int

const (
	FocusZoneMenu FocusZone = iota
	FocusZoneSubmenu
	FocusZoneActionPanel
)

func main() {
	app := tview.NewApplication()

	actionPanel := tview.NewFlex().SetDirection(tview.FlexRow)
	actionText := tview.NewTextView().
		SetText("Select an action from the submenu").
		SetDynamicColors(true).
		SetRegions(true).
		SetWrap(true)
	actionText.SetBorder(true).SetTitle("Action Panel")

	actionPanel.AddItem(actionText, 0, 1, false)

	// Top half horizontal flex: main menu and submenu each half width
	var topHalf = tview.NewFlex().SetDirection(tview.FlexColumn)

	// Create menu and submenus
	menu, submenus := createMenu(app, actionText)

	topHalf.AddItem(menu, 0, 1, true)

	// Focus management
	var currentFocusZone FocusZone = FocusZoneMenu

	var currentSubmenu tview.Primitive

	// Function to show only the selected submenu directly
	showSubmenu := func(name string) {
		if currentSubmenu != nil {
			topHalf.RemoveItem(currentSubmenu)
		}
		if submenu, ok := submenus[name]; ok {
			currentSubmenu = submenu
			if list, ok := currentSubmenu.(*tview.List); ok {
				list.SetTitle(name)
			}
			topHalf.AddItem(submenu, 0, 1, false)
		} else {
			currentSubmenu = nil
		}
	}

	// Set selected function for main menu items to show submenu and set focus
	menu.SetSelectedFunc(func(index int, mainText string, secondaryText string, shortcut rune) {
		if submenu, ok := submenus[mainText]; ok {
			// Show selected submenu
			showSubmenu(mainText)
			// Set focus to submenu
			app.SetFocus(submenu)
			currentFocusZone = FocusZoneSubmenu
		}
	})

	// Add borders and titles to menus
	menu.SetBorder(true).SetTitle("Main Menu")
	for _, submenu := range submenus {
		submenu.SetBorder(true).SetTitle("Submenu")
	}

	// Bottom half is action panel
	bottomHalf := tview.NewFlex().SetDirection(tview.FlexRow).
		AddItem(actionPanel, 0, 1, false)

	// Left panel for generic instructions
	leftStatus := tview.NewTextView().
		SetText("Tab or arrow keys: Move focus between menus").
		SetTextAlign(tview.AlignLeft)
	leftStatus.SetBackgroundColor(tcell.ColorDarkGray)

	// Right panel for context-aware status messages
	rightStatus := tview.NewTextView().
		SetText("").
		SetTextAlign(tview.AlignRight)
	rightStatus.SetBackgroundColor(tcell.ColorDarkGray)

	// Status bar flex container
	statusBar := tview.NewFlex().
		AddItem(leftStatus, 0, 1, false).
		AddItem(rightStatus, 0, 1, false)

	// Main layout vertical flex: top half and bottom half
	mainLayout := tview.NewFlex().SetDirection(tview.FlexRow)

	// Function to update layout based on terminal size
	updateLayout := func(screen tcell.Screen) {
		cols, rows := screen.Size()

		// Show current terminal size in rightStatus
		rightStatus.SetText(fmt.Sprintf("Terminal size: %dx%d", cols, rows))

		// Calculate half heights
		topHeight := rows / 2
		bottomHeight := rows - topHeight - 1 // subtract status bar height

		mainLayout.Clear()
		mainLayout.AddItem(topHalf, topHeight, 0, true)
		mainLayout.AddItem(bottomHalf, bottomHeight, 0, false)
		mainLayout.AddItem(statusBar, 1, 0, false)
	}

	// Initial layout update
	app.SetBeforeDrawFunc(func(screen tcell.Screen) bool {
		updateLayout(screen)
		return false
	})

	// Colors for focused and unfocused states
	focusedBorderColor := tcell.ColorYellow
	unfocusedBorderColor := tcell.ColorGray

	// Helper function to update colors based on focus
	updateFocusColors := func(focused tview.Primitive) {
		// Update main menu
		if menu == focused {
			menu.SetBorderColor(focusedBorderColor).SetTitleColor(focusedBorderColor)
		} else {
			menu.SetBorderColor(unfocusedBorderColor).SetTitleColor(unfocusedBorderColor)
		}

		// Update submenus
		for _, submenu := range submenus {
			if submenu == focused {
				submenu.SetBorderColor(focusedBorderColor).SetTitleColor(focusedBorderColor)
			} else {
				submenu.SetBorderColor(unfocusedBorderColor).SetTitleColor(unfocusedBorderColor)
			}
		}
	}

	// Helper to set focus and update colors
	setFocus := func(zone FocusZone) {
		currentFocusZone = zone
		switch zone {
		case FocusZoneMenu:
			app.SetFocus(menu)
			updateFocusColors(menu)
		case FocusZoneSubmenu:
			if currentSubmenu != nil {
				app.SetFocus(currentSubmenu)
				updateFocusColors(currentSubmenu)
			} else {
				app.SetFocus(menu)
				updateFocusColors(menu)
			}
		case FocusZoneActionPanel:
			app.SetFocus(actionPanel)
			// No border color update for action panel
		}
	}

	// Variable to track if action UI is active
	var actionUIActive bool
	var actionUIPrimitive tview.Primitive

	// Function to clear action UI and return focus to menu
	clearActionUI := func() {
		actionPanel.Clear()
		actionText.SetText("Select an action from the submenu")
		actionPanel.AddItem(actionText, 0, 1, false)
		setFocus(FocusZoneMenu)
		actionUIActive = false
		actionUIPrimitive = nil
	}

	// New function wrapping createSaveChangesSubmenu to inject cancel callback
	createSaveChangesSubmenuWithCancel := func(app *tview.Application, actionPanel *tview.Flex, actionText *tview.TextView, onCancel func()) *tview.List {
		list := createSaveChangesSubmenu(actionText)
		list.SetSelectedFunc(func(index int, mainText string, secondaryText string, shortcut rune) {
			if mainText == "Save Now" {
				actionUIActive = true
				actionUIPrimitive = save.ShowSaveUI(actionPanel, app, actionText, onCancel)
				if actionUIPrimitive != nil {
					app.SetFocus(actionUIPrimitive)
				}
			}
		})
		return list
	}

	// Replace submenu creation for SaveChangesKey
	submenus = map[string]*tview.List{
		SaveChangesKey: createSaveChangesSubmenuWithCancel(app, actionPanel, actionText, clearActionUI),
		CheckFilesKey:  createCheckFilesSubmenu(actionText),
		BranchesKey:    createBranchesSubmenu(actionText),
		SyncChangesKey: createSyncChangesSubmenu(actionText),
		SettingsKey:    createSettingsSubmenu(actionText),
	}

	// Set selected function for main menu items to show submenu and set focus
	menu.SetSelectedFunc(func(index int, mainText string, secondaryText string, shortcut rune) {
		if submenu, ok := submenus[mainText]; ok {
			// Show selected submenu
			showSubmenu(mainText)
			// Set focus to submenu
			app.SetFocus(submenu)
			currentFocusZone = FocusZoneSubmenu
		}
	})

	// Set input capture to delegate keys to action UI when active
	app.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		if actionUIActive && actionUIPrimitive != nil {
			// Let action UI handle all keys
			return nil
		}

		switch event.Key() {
		case tcell.KeyTab:
			if event.Modifiers()&tcell.ModShift != 0 {
				// Shift+Tab: cycle focus backward between menu and submenu only
				if currentFocusZone == FocusZoneMenu {
					setFocus(FocusZoneSubmenu)
				} else {
					setFocus(FocusZoneMenu)
				}
			} else {
				// Tab: cycle focus forward between menu and submenu only
				if currentFocusZone == FocusZoneSubmenu {
					setFocus(FocusZoneMenu)
				} else {
					setFocus(FocusZoneSubmenu)
				}
			}
			return nil

		case tcell.KeyEsc:
			// Esc to return focus to menu and reset action panel to initial default state
			clearActionUI()
			return nil

		default:
			return event
		}
	})

	// Initialize colors and focus
	setFocus(FocusZoneMenu)

	if err := app.SetRoot(mainLayout, true).SetFocus(menu).Run(); err != nil {
		panic(err)
	}
}
