package main

import (
	"github.com/rivo/tview"
)

const (
	SaveChangesKey = "Save Changes"
	CheckFilesKey  = "Check Files"
	BranchesKey    = "Branches (Work Areas)"
	SyncChangesKey = "Sync Changes"
	SettingsKey    = "Settings"
)

var helpTexts = map[string]string{
	SaveChangesKey: `Save Changes help:

"Save Now" means to save your current work safely.
Example: You edited files and want to save a snapshot.

"Fix Last Save" lets you change your last saved snapshot.
Example: You forgot to add a file, so you fix the last save.

"Undo Changes" lets you discard changes you made since last save.
Example: You made a mistake and want to go back to the last saved state.

"View History" shows all your saved snapshots.

"Search History" helps find a saved snapshot by keyword.`,

	CheckFilesKey: `Check Files help:

"Show Changes" lets you see what files have changed since last save.
Example: You want to know which files you edited.

"View File Differences" shows line-by-line changes in a file.
Example: See exactly what you changed in a file.`,

	BranchesKey: `Branches (Work Areas) help:

"Show Branches" lists all versions of your work.

"New Branch" starts a new version to work on.
Example: You want to try a new feature without changing main work.

"Remove Branch" deletes a version you no longer need.

"Merge Branch" combines changes from one version into another.
Example: You finished a feature in a separate branch and want to add it to your main work.

"Rename Branch" lets you rename a branch for clarity.

A branch is like a separate workspace for your changes.`,

	SyncChangesKey: `Sync Changes help:

"Send Updates" sends your saved work to the central place.

"Get Updates" gets work saved by others.

"Check for Updates" checks if others have new work.

"Sync All" sends your work and gets others' work to keep up to date.`,

	SettingsKey: `Settings help:

"Set Name" sets your name for saved work.

"Set Email" sets your email for saved work.

"Other Options" lets you change extra settings.`,
}

func createSaveChangesSubmenu(actionPanel *tview.TextView) *tview.List {
	return tview.NewList().
		AddItem("Save Now", "Save your current work", 'n', nil).
		AddItem("Fix Last Save", "Change your last saved work", 'a', nil).
		AddItem("Undo Changes", "Discard changes since last save", 'u', nil).
		AddItem("View History", "See past saved work", 'v', nil).
		AddItem("Search History", "Find saved work by keyword", 's', nil).
		AddItem("Help", "What is saving?", 'h', func() {
			actionPanel.SetText(helpTexts[SaveChangesKey])
		})
}

func createCheckFilesSubmenu(actionPanel *tview.TextView) *tview.List {
	return tview.NewList().
		AddItem("Show Changes", "See what files changed", 's', nil).
		AddItem("View File Differences", "See line-by-line changes", 'd', nil).
		AddItem("Help", "What is checking files?", 'h', func() {
			actionPanel.SetText(helpTexts[CheckFilesKey])
		})
}

func createBranchesSubmenu(actionPanel *tview.TextView) *tview.List {
	return tview.NewList().
		AddItem("Show Branches", "See all versions of your work", 'l', nil).
		AddItem("New Branch", "Start a new version of your work", 'c', nil).
		AddItem("Remove Branch", "Delete a version of your work", 'd', nil).
		AddItem("Merge Branch", "Combine changes from one version into another", 'm', nil).
		AddItem("Rename Branch", "Rename a branch", 'r', nil).
		AddItem("Help", "What is a branch?", 'h', func() {
			actionPanel.SetText(helpTexts[BranchesKey])
		})
}

func createSyncChangesSubmenu(actionPanel *tview.TextView) *tview.List {
	return tview.NewList().
		AddItem("Send Updates", "Send your work to the central place", 'p', nil).
		AddItem("Get Updates", "Get work from others", 'l', nil).
		AddItem("Check for Updates", "See if others have new work", 'f', nil).
		AddItem("Sync All", "Send and get updates", 's', nil).
		AddItem("Help", "What is syncing?", 'h', func() {
			actionPanel.SetText(helpTexts[SyncChangesKey])
		})
}

func createSettingsSubmenu(actionPanel *tview.TextView) *tview.List {
	return tview.NewList().
		AddItem("Set Name", "Your name for saved work", 'u', nil).
		AddItem("Set Email", "Your email for saved work", 'e', nil).
		AddItem("Other Options", "Extra settings", 'c', nil).
		AddItem("Help", "Settings help", 'h', func() {
			actionPanel.SetText(helpTexts[SettingsKey])
		})
}

func createMainMenu(app *tview.Application) *tview.List {
	return tview.NewList().
		AddItem(SaveChangesKey, "Save your work safely", 'c', nil).
		AddItem(CheckFilesKey, "See what changed in your files", 's', nil).
		AddItem(BranchesKey, "Manage different versions of your work", 'b', nil).
		AddItem(SyncChangesKey, "Keep your work up to date", 'y', nil).
		AddItem(SettingsKey, "Set your name and options", 'o', nil).
		AddItem("Exit", "Close the program", 'q', func() {
			app.Stop()
		})
}

func createMenu(app *tview.Application, actionPanel *tview.TextView) (*tview.List, map[string]*tview.List) {
	submenus := map[string]*tview.List{
		SaveChangesKey: createSaveChangesSubmenu(actionPanel),
		CheckFilesKey:  createCheckFilesSubmenu(actionPanel),
		BranchesKey:    createBranchesSubmenu(actionPanel),
		SyncChangesKey: createSyncChangesSubmenu(actionPanel),
		SettingsKey:    createSettingsSubmenu(actionPanel),
	}

	for _, submenu := range submenus {
		submenu.SetBorder(true).SetTitle("Submenu")
	}

	menu := createMainMenu(app)
	menu.SetBorder(true).SetTitle("Menu").SetTitleAlign(tview.AlignLeft)

	return menu, submenus
}
