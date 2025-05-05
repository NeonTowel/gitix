package save

import (
	"bytes"
	"os/exec"
	"strings"

	"github.com/gdamore/tcell/v2"
	"github.com/rivo/tview"
)

// GetChangedFiles returns a list of changed files (staged and unstaged) in the git repo
func GetChangedFiles() ([]string, error) {
	cmd := exec.Command("git", "status", "--porcelain")
	var out bytes.Buffer
	cmd.Stdout = &out
	if err := cmd.Run(); err != nil {
		return nil, err
	}
	lines := strings.Split(out.String(), "\n")
	files := []string{}
	for _, line := range lines {
		if len(line) < 4 {
			continue
		}
		// line format: XY filename
		filename := strings.TrimSpace(line[3:])
		files = append(files, filename)
	}
	return files, nil
}

// StageFiles stages the given files for commit
func StageFiles(files []string) error {
	args := append([]string{"add"}, files...)
	cmd := exec.Command("git", args...)
	return cmd.Run()
}

// Commit commits staged changes with the given commit message
func Commit(message string) error {
	cmd := exec.Command("git", "commit", "-m", message)
	return cmd.Run()
}

// SaveNow is deprecated, use StageFiles and Commit instead
func SaveNow() error {
	return nil
}

// ShowSaveUI updates the given container (action panel) with the save UI
func ShowSaveUI(container *tview.Flex, app *tview.Application, actionPanel *tview.TextView, onCancel func()) tview.Primitive {
	files, err := GetChangedFiles()
	if err != nil {
		actionPanel.SetText("Error getting changed files: " + err.Error())
		return nil
	}

	// Clear container
	container.Clear()

	// Create checkbox list for files
	fileList := tview.NewList().ShowSecondaryText(false)
	selectedFiles := map[int]string{}
	for i, f := range files {
		fileName := f
		fileList.AddItem(f, "", 0, func() {
			// Toggle selection
			if _, ok := selectedFiles[i]; ok {
				delete(selectedFiles, i)
				fileList.SetItemText(i, fileName, "")
			} else {
				selectedFiles[i] = fileName
				fileList.SetItemText(i, "[x] "+fileName, "")
			}
		})
	}

	// Commit message input
	commitInput := tview.NewInputField().
		SetLabel("Commit message (conventional commits style): ").
		SetFieldWidth(50)

	// Submit button
	submitButton := tview.NewButton("Commit").SetSelectedFunc(func() {
		if len(selectedFiles) == 0 {
			actionPanel.SetText("No files selected to commit.")
			return
		}
		message := commitInput.GetText()
		if strings.TrimSpace(message) == "" {
			actionPanel.SetText("Commit message cannot be empty.")
			return
		}

		// Stage files
		filesToStage := []string{}
		for _, f := range selectedFiles {
			filesToStage = append(filesToStage, f)
		}
		err := StageFiles(filesToStage)
		if err != nil {
			actionPanel.SetText("Error staging files: " + err.Error())
			return
		}

		// Commit
		err = Commit(message)
		if err != nil {
			actionPanel.SetText("Error committing: " + err.Error())
			return
		}

		actionPanel.SetText("Commit successful.")
	})

	cancelFunc := func() {
		// Clear action panel
		container.Clear()
		actionPanel.SetText("")
		// Call onCancel callback to notify main app
		if onCancel != nil {
			onCancel()
		}
	}

	cancelButton := tview.NewButton("Cancel (Esc)").SetSelectedFunc(cancelFunc)

	// Layout inside container
	formFlex := tview.NewFlex().SetDirection(tview.FlexRow).
		AddItem(fileList, 0, 1, true).
		AddItem(commitInput, 1, 0, false).
		AddItem(submitButton, 1, 0, false).
		AddItem(cancelButton, 1, 0, false)

	container.AddItem(formFlex, 0, 1, true)

	// Capture all keys inside action UI
	formFlex.SetInputCapture(func(event *tcell.EventKey) *tcell.EventKey {
		switch event.Key() {
		case tcell.KeyEsc:
			cancelFunc()
			return nil
		default:
			return event
		}
	})

	return formFlex
}
