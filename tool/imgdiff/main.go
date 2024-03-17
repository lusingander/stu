package main

import (
	"context"
	"flag"
	"fmt"
	"image"
	"image/png"
	"io/fs"
	"log"
	"os"
	"path/filepath"
	"strings"

	"github.com/google/subcommands"
	"github.com/logrusorgru/aurora/v4"
	"github.com/n7olkachev/imgdiff/pkg/imgdiff"
	"golang.org/x/exp/slices"
)

type testCmd struct {
	baseDirPath   string
	targetDirPath string
	outputDirPath string
}

func (*testCmd) Name() string { return "test" }

func (*testCmd) Synopsis() string { return "Test image diff" }

func (*testCmd) Usage() string { return "test -base <dir> -target <dir> -out <dir>\n" }

func (cmd *testCmd) SetFlags(f *flag.FlagSet) {
	f.StringVar(&cmd.baseDirPath, "base", "", "base directory path")
	f.StringVar(&cmd.targetDirPath, "target", "", "target directory path")
	f.StringVar(&cmd.outputDirPath, "out", "", "output directory path")
}

func (cmd *testCmd) Execute(_ context.Context, f *flag.FlagSet, args ...any) subcommands.ExitStatus {
	if err := cmd.run(); err != nil {
		log.Println(err)
		return subcommands.ExitFailure
	}
	return subcommands.ExitSuccess
}

func (cmd *testCmd) run() error {
	baseFiles, err := os.ReadDir(cmd.baseDirPath)
	if err != nil {
		return err
	}
	targetFiles, err := os.ReadDir(cmd.targetDirPath)
	if err != nil {
		return err
	}

	if err := cleanOutputDir(cmd.outputDirPath); err != nil {
		return err
	}

	files := make(map[string]struct{})
	for _, baseFile := range baseFiles {
		name := baseFile.Name()
		if strings.HasSuffix(name, ".png") {
			files[name] = struct{}{}
		}
	}
	for _, targetFile := range targetFiles {
		name := targetFile.Name()
		if strings.HasSuffix(name, ".png") {
			files[name] = struct{}{}
		}
	}

	results := make([]result, 0)

	for file := range files {
		if !existFile(file, baseFiles) {
			results = append(results, result{resultNew, file})
			continue
		}
		if !existFile(file, targetFiles) {
			results = append(results, result{resultDeleted, file})
			continue
		}

		baseImage, err := openImage(cmd.baseDirPath, file)
		if err != nil {
			return err
		}
		targetImage, err := openImage(cmd.targetDirPath, file)
		if err != nil {
			return err
		}

		opts := &imgdiff.Options{
			Threshold: 0.1,
			DiffImage: true,
		}
		diff := imgdiff.Diff(baseImage, targetImage, opts)
		if diff.Equal {
			results = append(results, result{resultSuccess, file})
		} else {
			results = append(results, result{resultFailure, file})
			if err := createOutputDir(cmd.outputDirPath); err != nil {
				return err
			}
			if err := outputImage(cmd.outputDirPath, file, diff.Image); err != nil {
				return err
			}
		}
	}

	slices.SortFunc(results, func(r1, r2 result) bool {
		if r1.tp == r2.tp {
			return r1.file < r2.file
		}
		return resultTpOrder(r1.tp) < resultTpOrder(r2.tp)
	})
	for _, r := range results {
		switch r.tp {
		case resultNew:
			fmt.Printf("%s: %s\n", r.file, aurora.Blue(resultNew))
		case resultDeleted:
			fmt.Printf("%s: %s\n", r.file, aurora.Yellow(resultDeleted))
		case resultSuccess:
			fmt.Printf("%s: %s\n", r.file, aurora.Green(resultSuccess))
		case resultFailure:
			fmt.Printf("%s: %s\n", r.file, aurora.Red(resultFailure))
		}
	}

	return nil
}

const (
	resultNew     = "New"
	resultDeleted = "Deleted"
	resultSuccess = "Success"
	resultFailure = "Failure"
)

func resultTpOrder(tp string) int {
	switch tp {
	case resultNew:
		return 0
	case resultDeleted:
		return 1
	case resultSuccess:
		return 2
	case resultFailure:
		return 3
	}
	return 0
}

type result struct {
	tp   string
	file string
}

func existFile(target string, es []fs.DirEntry) bool {
	for _, e := range es {
		if target == e.Name() {
			return true
		}
	}
	return false
}

func cleanOutputDir(dir string) error {
	return os.RemoveAll(dir)
}

func createOutputDir(dir string) error {
	return os.MkdirAll(dir, os.ModePerm)
}

func outputImage(dir, name string, img image.Image) error {
	path := filepath.Join(dir, name)
	dst, err := os.Create(path)
	if err != nil {
		return err
	}
	if err := png.Encode(dst, img); err != nil {
		return err
	}
	return nil
}

func openImage(dir, name string) (image.Image, error) {
	path := filepath.Join(dir, name)

	file, err := os.Open(path)
	if err != nil {
		return nil, err
	}
	defer file.Close()

	img, _, err := image.Decode(file)
	return img, err
}

func main() {
	subcommands.Register(subcommands.HelpCommand(), "")
	subcommands.Register(subcommands.CommandsCommand(), "")
	subcommands.Register(subcommands.FlagsCommand(), "")
	subcommands.Register(&testCmd{}, "")
	flag.Parse()
	ctx := context.Background()
	os.Exit(int(subcommands.Execute(ctx)))
}
