package main

import (
	"github.com/gdamore/tcell/v2"
	"github.com/rivo/tview"
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

	// Content panel holds submenu on left and action panel on right
	content := tview.NewFlex()

	// Function to show only the selected submenu in content
	showSubmenu := func(name string) {
		content.Clear()
		if submenu, ok := submenus[name]; ok {
			content.AddItem(submenu, 0, 1, true)
			content.AddItem(actionPanel, 0, 3, false)
		}
	}

	menu.SetSelectedFunc(func(index int, mainText string, secondaryText string, shortcut rune) {
		if mainText == "Exit" {
			app.Stop()
		} else {
			showSubmenu(mainText)
		}
	})

	// Initially show the first submenu
	showSubmenu("Save Changes")

	flex := tview.NewFlex().
		AddItem(menu, 0, 1, true).
		AddItem(content, 0, 4, false)

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

	// Main layout flex with vertical direction
	mainLayout := tview.NewFlex().SetDirection(tview.FlexRow).
		AddItem(flex, 0, 1, true).
		AddItem(statusBar, 1, 0, false) // Set height to 1 explicitly

	// Keybindings to move focus between menu and submenu only
	app.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		switch event.Key() {
		case tcell.KeyTab:
			if app.GetFocus() == menu {
				mainText, _ := menu.GetItemText(menu.GetCurrentItem())
				showSubmenu(mainText)
				if submenu, ok := submenus[mainText]; ok {
					app.SetFocus(submenu)
				}
				return nil
			} else {
				app.SetFocus(menu)
				return nil
			}
		case tcell.KeyRight:
			if app.GetFocus() == menu {
				mainText, _ := menu.GetItemText(menu.GetCurrentItem())
				showSubmenu(mainText)
				if submenu, ok := submenus[mainText]; ok {
					app.SetFocus(submenu)
				}
				return nil
			}
		case tcell.KeyEnter:
			if app.GetFocus() == menu {
				index := menu.GetCurrentItem()
				mainText, secondaryText := menu.GetItemText(index)
				menu.GetSelectedFunc()(index, mainText, secondaryText, 0)
				return nil
			}
		case tcell.KeyLeft:
			if focus := app.GetFocus(); focus != nil {
				for _, submenu := range submenus {
					if focus == submenu {
						app.SetFocus(menu)
						return nil
					}
				}
			}
		}
		return event
	})

	if err := app.SetRoot(mainLayout, true).Run(); err != nil {
		panic(err)
	}
}
