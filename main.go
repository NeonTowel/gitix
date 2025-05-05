package main

import (
	"github.com/gdamore/tcell/v2"
	"github.com/rivo/tview"
)

func main() {
	app := tview.NewApplication()

	menu := tview.NewList().
		AddItem("Commits", "Create and manage commits", 'c', nil).
		AddItem("Status", "View git status", 's', nil).
		AddItem("Branch Management", "Manage branches", 'b', nil).
		AddItem("Sync", "Sync changes with remote", 'y', nil).
		AddItem("Configuration", "Configure user and options", 'o', nil).
		AddItem("Quit", "Press to exit", 'q', func() {
			app.Stop()
		})

	menu.SetBorder(true).SetTitle("Menu").SetTitleAlign(tview.AlignLeft)

	// Submenu lists for each main menu item
	commitSubmenu := tview.NewList().
		AddItem("New Commit", "Create a new commit", 'n', nil).
		AddItem("Amend Commit", "Amend last commit", 'a', nil).
		AddItem("View Log", "Show commit log", 'v', nil).
		AddItem("Search History", "Search commits", 's', nil)

	configurationSubmenu := tview.NewList().
		AddItem("User Name", "Set user name", 'u', nil).
		AddItem("User Email", "Set user email", 'e', nil).
		AddItem("Custom Options", "Set custom options", 'c', nil)

	statusSubmenu := tview.NewList().
		AddItem("Show Status", "Show git status", 's', nil)

	branchSubmenu := tview.NewList().
		AddItem("List Branches", "List all branches", 'l', nil).
		AddItem("Create Branch", "Create a new branch", 'c', nil).
		AddItem("Delete Branch", "Delete a branch", 'd', nil)

	syncSubmenu := tview.NewList().
		AddItem("Sync", "Sync changes with remote", 's', nil).
		AddItem("Fetch", "Fetch changes from remote", 'f', nil).
		AddItem("Push", "Push changes to remote", 'p', nil).
		AddItem("Pull", "Pull changes from remote", 'l', nil)

	submenus := map[string]*tview.List{
		"Commits":           commitSubmenu,
		"Status":            statusSubmenu,
		"Branch Management": branchSubmenu,
		"Sync":              syncSubmenu,
		"Configuration":     configurationSubmenu,
	}

	for _, submenu := range submenus {
		submenu.SetBorder(true).SetTitle("Submenu")
	}

	// Action panel to the right of submenu
	actionPanel := tview.NewTextView().
		SetText("Select an action from the submenu").
		SetDynamicColors(true).
		SetRegions(true).
		SetWrap(true)
	actionPanel.SetBorder(true).SetTitle("Action Panel")

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

	menu.SetChangedFunc(func(index int, mainText string, secondaryText string, shortcut rune) {
		showSubmenu(mainText)
	})

	// Initially show the first submenu
	showSubmenu("Commits")

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
		case tcell.KeyRight, tcell.KeyEnter:
			if app.GetFocus() == menu {
				mainText, _ := menu.GetItemText(menu.GetCurrentItem())
				showSubmenu(mainText)
				if submenu, ok := submenus[mainText]; ok {
					app.SetFocus(submenu)
				}
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
