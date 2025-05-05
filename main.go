package main

import (
	"github.com/gdamore/tcell/v2"
	"github.com/rivo/tview"
)

func main() {
	app := tview.NewApplication()

	menu := tview.NewList().
		AddItem("Save Changes", "Save your work safely", 'c', nil).
		AddItem("Check Files", "See what changed in your files", 's', nil).
		AddItem("Branches (Work Areas)", "Manage different versions of your work", 'b', nil).
		AddItem("Sync Changes", "Keep your work up to date", 'y', nil).
		AddItem("Settings", "Set your name and options", 'o', nil).
		AddItem("Exit", "Close the program", 'q', func() {
			app.Stop()
		})

	menu.SetBorder(true).SetTitle("Menu").SetTitleAlign(tview.AlignLeft)

	// Action panel to the right of submenu
	actionPanel := tview.NewTextView().
		SetText("Select an action from the submenu").
		SetDynamicColors(true).
		SetRegions(true).
		SetWrap(true)
	actionPanel.SetBorder(true).SetTitle("Action Panel")

	// Help text constants
	const saveHelp = `Save Changes help:

"Save Now" means to save your current work safely.
Example: You edited files and want to save a snapshot.

"Fix Last Save" lets you change your last saved snapshot.
Example: You forgot to add a file, so you fix the last save.

"Undo Changes" lets you discard changes you made since last save.
Example: You made a mistake and want to go back to the last saved state.

"View History" shows all your saved snapshots.

"Search History" helps find a saved snapshot by keyword.`

	const checkFilesHelp = `Check Files help:

"Show Changes" lets you see what files have changed since last save.
Example: You want to know which files you edited.

"View File Differences" shows line-by-line changes in a file.
Example: See exactly what you changed in a file.`

	const branchesHelp = `Branches (Work Areas) help:

"Show Branches" lists all versions of your work.

"New Branch" starts a new version to work on.
Example: You want to try a new feature without changing main work.

"Remove Branch" deletes a version you no longer need.

"Merge Branch" combines changes from one version into another.
Example: You finished a feature in a separate branch and want to add it to your main work.

"Rename Branch" lets you rename a branch for clarity.

A branch is like a separate workspace for your changes.`

	const syncHelp = `Sync Changes help:

"Send Updates" sends your saved work to the central place.

"Get Updates" gets work saved by others.

"Check for Updates" checks if others have new work.

"Sync All" sends your work and gets others' work to keep up to date.`

	const settingsHelp = `Settings help:

"Set Name" sets your name for saved work.

"Set Email" sets your email for saved work.

"Other Options" lets you change extra settings.`

	saveChangesSubmenu := tview.NewList().
		AddItem("Save Now", "Save your current work", 'n', nil).
		AddItem("Fix Last Save", "Change your last saved work", 'a', nil).
		AddItem("Undo Changes", "Discard changes since last save", 'u', nil).
		AddItem("View History", "See past saved work", 'v', nil).
		AddItem("Search History", "Find saved work by keyword", 's', nil).
		AddItem("Help", "What is saving?", 'h', func() {
			actionPanel.SetText(saveHelp)
		})

	checkFilesSubmenu := tview.NewList().
		AddItem("Show Changes", "See what files changed", 's', nil).
		AddItem("View File Differences", "See line-by-line changes", 'd', nil).
		AddItem("Help", "What is checking files?", 'h', func() {
			actionPanel.SetText(checkFilesHelp)
		})

	branchesSubmenu := tview.NewList().
		AddItem("Show Branches", "See all versions of your work", 'l', nil).
		AddItem("New Branch", "Start a new version of your work", 'c', nil).
		AddItem("Remove Branch", "Delete a version of your work", 'd', nil).
		AddItem("Merge Branch", "Combine changes from one version into another", 'm', nil).
		AddItem("Rename Branch", "Rename a branch", 'r', nil).
		AddItem("Help", "What is a branch?", 'h', func() {
			actionPanel.SetText(branchesHelp)
		})

	syncChangesSubmenu := tview.NewList().
		AddItem("Send Updates", "Send your work to the central place", 'p', nil).
		AddItem("Get Updates", "Get work from others", 'l', nil).
		AddItem("Check for Updates", "See if others have new work", 'f', nil).
		AddItem("Sync All", "Send and get updates", 's', nil).
		AddItem("Help", "What is syncing?", 'h', func() {
			actionPanel.SetText(syncHelp)
		})

	settingsSubmenu := tview.NewList().
		AddItem("Set Name", "Your name for saved work", 'u', nil).
		AddItem("Set Email", "Your email for saved work", 'e', nil).
		AddItem("Other Options", "Extra settings", 'c', nil).
		AddItem("Help", "Settings help", 'h', func() {
			actionPanel.SetText(settingsHelp)
		})

	submenus := map[string]*tview.List{
		"Save Changes":          saveChangesSubmenu,
		"Check Files":           checkFilesSubmenu,
		"Branches (Work Areas)": branchesSubmenu,
		"Sync Changes":          syncChangesSubmenu,
		"Settings":              settingsSubmenu,
	}

	for _, submenu := range submenus {
		submenu.SetBorder(true).SetTitle("Submenu")
	}

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
