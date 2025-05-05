package main

import (
	"fmt"

	"github.com/gdamore/tcell/v2"
	"github.com/rivo/tview"
)

const (
	minWidth  = 110
	minHeight = 30
)

func main() {
	app := tview.NewApplication()

	actionPanel := tview.NewTextView().
		SetText("Select an action from the submenu").
		SetDynamicColors(true).
		SetRegions(true).
		SetWrap(true)
	actionPanel.SetBorder(true).SetTitle("Action Panel")

	menu, submenus := createMenu(app, actionPanel)

	// Add borders and titles to menus
	menu.SetBorder(true).SetTitle("Main Menu")
	for _, submenu := range submenus {
		submenu.SetBorder(true).SetTitle("Submenu")
	}

	// Top half horizontal flex: main menu and submenu each half width
	topHalf := tview.NewFlex().SetDirection(tview.FlexColumn).
		AddItem(menu, 0, 1, true)

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

	// Initially show the first submenu
	showSubmenu("Save Changes")

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

	// Set input capture to update colors on focus change
	app.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		// Call existing key handling code first
		switch event.Key() {
		case tcell.KeyTab:
			if app.GetFocus() == menu {
				mainText, _ := menu.GetItemText(menu.GetCurrentItem())
				if mainText != "Exit" {
					showSubmenu(mainText)
					app.SetFocus(submenus[mainText])
					updateFocusColors(submenus[mainText])
				}
				return nil
			} else {
				app.SetFocus(menu)
				updateFocusColors(menu)
				return nil
			}
		case tcell.KeyRight:
			if app.GetFocus() == menu {
				mainText, _ := menu.GetItemText(menu.GetCurrentItem())
				if mainText != "Exit" {
					showSubmenu(mainText)
					app.SetFocus(submenus[mainText])
					updateFocusColors(submenus[mainText])
				}
				return nil
			}
		case tcell.KeyEnter:
			if app.GetFocus() == menu {
				index := menu.GetCurrentItem()
				if index >= 0 {
					mainText, secondaryText := menu.GetItemText(index)
					if mainText == "Exit" {
						app.Stop()
					} else {
						showSubmenu(mainText)
						app.SetFocus(submenus[mainText])
						updateFocusColors(submenus[mainText])
						if menu.GetSelectedFunc() != nil {
							menu.GetSelectedFunc()(index, mainText, secondaryText, 0)
						}
					}
				}
				return nil
			}
		case tcell.KeyLeft:
			if focus := app.GetFocus(); focus != nil {
				if focus != menu {
					app.SetFocus(menu)
					updateFocusColors(menu)
					return nil
				}
			}
		}
		return event
	})

	// Initialize colors
	updateFocusColors(menu)

	if err := app.SetRoot(mainLayout, true).SetFocus(menu).Run(); err != nil {
		panic(err)
	}
}
