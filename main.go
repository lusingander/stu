package main

import (
	"fmt"
	"io"
	"log"
	"os"

	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
	"github.com/lusingander/stu/internal/aws"
	"github.com/mattn/go-runewidth"
)

var (
	listStyle = lipgloss.NewStyle().
			MarginTop(1).
			BorderStyle(lipgloss.NormalBorder()).
			BorderForeground(lipgloss.Color("63")).
			BorderTop(true)

	itemStyle = lipgloss.NewStyle().
			PaddingLeft(2)

	selectedItemStyle = lipgloss.NewStyle().
				PaddingLeft(0).
				Foreground(lipgloss.Color("170"))

	breadcrumbStyle = lipgloss.NewStyle().
			PaddingLeft(2).
			Height(1)
)

type model struct {
	list list.Model

	client      *aws.S3Client
	bucket      string
	breadcrumbs []*aws.ObjectItem
}

type listItem interface {
	Text() string
}

type itemDelegate struct{}

func (d itemDelegate) Height() int {
	return 1
}

func (d itemDelegate) Spacing() int {
	return 0
}

func (d itemDelegate) Update(msg tea.Msg, m *list.Model) tea.Cmd {
	return nil
}

func (d itemDelegate) Render(w io.Writer, m list.Model, index int, item list.Item) {
	i, ok := item.(listItem)
	if !ok {
		return
	}

	str := i.Text()

	fn := itemStyle.Render
	if index == m.Index() {
		fn = func(s string) string {
			return selectedItemStyle.Render("> " + s)
		}
	}

	fmt.Fprint(w, fn(str))
}

func (m model) Init() tea.Cmd {
	return nil
}

func (m model) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		switch msg.String() {
		case "enter":
			switch i := m.list.SelectedItem().(type) {
			case *aws.BucketItem:
				bucket := i.BucketName()
				objs, err := m.client.ListObjects(bucket, "")
				if err != nil {
					return m, tea.Quit
				}
				items := make([]list.Item, len(objs))
				for i, obj := range objs {
					items[i] = obj
				}
				m.list.SetItems(items)
				m.list.ResetSelected()
				m.list.ResetFilter()
				m.bucket = bucket
			case *aws.ObjectItem:
				if i.Dir {
					objs, err := m.client.ListObjects(m.bucket, i.ObjectKey())
					if err != nil {
						return m, tea.Quit
					}
					items := make([]list.Item, len(objs))
					for i, obj := range objs {
						items[i] = obj
					}
					m.list.SetItems(items)
					m.list.ResetSelected()
					m.list.ResetFilter()
					m.breadcrumbs = append(m.breadcrumbs, i)
				}
			}
		case "backspace", "ctrl+h":
			switch m.list.SelectedItem().(type) {
			case *aws.BucketItem:
				// do nothing
			case *aws.ObjectItem:
				bl := len(m.breadcrumbs)
				if bl == 0 {
					buckets, err := m.client.ListBuckets()
					if err != nil {
						return m, tea.Quit
					}
					items := make([]list.Item, len(buckets))
					for i, bucket := range buckets {
						items[i] = bucket
					}
					m.list.SetItems(items)
					m.list.ResetSelected()
					m.list.ResetFilter()
					m.bucket = ""
				} else {
					var key string
					if bl == 1 {
						key = ""
					} else {
						key = m.breadcrumbs[bl-2].ObjectKey()
					}
					objs, err := m.client.ListObjects(m.bucket, key)
					if err != nil {
						return m, tea.Quit
					}
					items := make([]list.Item, len(objs))
					for i, obj := range objs {
						items[i] = obj
					}
					m.list.SetItems(items)
					m.list.ResetSelected()
					m.list.ResetFilter()
					m.breadcrumbs = m.breadcrumbs[:bl-1]
				}
			}
		}
	case tea.WindowSizeMsg:
		m.list.SetSize(msg.Width, msg.Height-3)
	}

	var cmd tea.Cmd
	m.list, cmd = m.list.Update(msg)
	return m, cmd
}

func (m model) viewBreadcrumb() string {
	sep := " > "
	s := "STU"
	if m.bucket != "" {
		s += sep
		s += m.bucket
		for _, b := range m.breadcrumbs {
			s += sep
			s += b.Filename()
		}
	}
	return s
}

func (m model) View() string {
	bc := breadcrumbStyle.Render(m.viewBreadcrumb())
	l := listStyle.Render(m.list.View())
	return bc + l
}

func setup() {
	runewidth.DefaultCondition = &runewidth.Condition{EastAsianWidth: false}
}

func run(args []string) error {
	setup()

	client, err := aws.NewS3Client()
	if err != nil {
		return err
	}
	buckets, err := client.ListBuckets()
	if err != nil {
		return err
	}

	items := make([]list.Item, len(buckets))
	for i, bucket := range buckets {
		items[i] = bucket
	}

	m := model{
		list:        list.NewModel(items, itemDelegate{}, 0, 0),
		client:      client,
		bucket:      "",
		breadcrumbs: make([]*aws.ObjectItem, 0),
	}
	m.list.SetShowTitle(false)
	m.list.Styles.TitleBar = lipgloss.Style{} // clear style...
	m.list.Styles.Title = lipgloss.Style{}
	m.list.SetShowStatusBar(false)

	p := tea.NewProgram(m)
	p.EnterAltScreen()

	return p.Start()
}

func main() {
	if err := run(os.Args); err != nil {
		log.Fatal(err)
	}
}
