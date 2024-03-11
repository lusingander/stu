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

	for _, baseFile := range baseFiles {
		if !strings.HasSuffix(baseFile.Name(), ".png") {
			continue
		}

		targetFile := findByName(baseFile, targetFiles)
		if targetFile == nil {
			fmt.Printf("%s: %s\n", baseFile.Name(), aurora.Blue("Not found"))
			continue
		}

		baseImage, err := openImage(cmd.baseDirPath, targetFile.Name())
		if err != nil {
			return err
		}
		targetImage, err := openImage(cmd.targetDirPath, targetFile.Name())
		if err != nil {
			return err
		}

		opts := &imgdiff.Options{
			Threshold: 0.1,
			DiffImage: true,
		}
		result := imgdiff.Diff(baseImage, targetImage, opts)
		if result.Equal {
			fmt.Printf("%s: %s\n", baseFile.Name(), aurora.Green("Success"))
		} else {
			fmt.Printf("%s: %s\n", baseFile.Name(), aurora.Red("Failure"))
			if err := createOutputDir(cmd.outputDirPath); err != nil {
				return err
			}
			if err := outputImage(cmd.outputDirPath, baseFile.Name(), result.Image); err != nil {
				return err
			}
		}
	}

	return nil
}

func findByName(target fs.DirEntry, es []fs.DirEntry) fs.DirEntry {
	for _, e := range es {
		if target.Name() == e.Name() {
			return e
		}
	}
	return nil
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
