package main

import (
	"bytes"
	"context"
	"errors"
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"

	"github.com/google/subcommands"
)

const (
	vhsVersion = "0.7.1"
)

func checkVhs() error {
	var bufOut, bufErr bytes.Buffer
	cmd := exec.Command("vhs", "--version")
	cmd.Stdout = &bufOut
	cmd.Stderr = &bufErr
	versionStr := "vhs version v" + vhsVersion
	if err := cmd.Run(); err != nil || !strings.HasPrefix(bufOut.String(), versionStr) {
		return fmt.Errorf("vhs %s is not available. %v", vhsVersion, err)
	}
	return nil
}

func readTape(tapefile string, variables map[string]string) (string, error) {
	bytes, err := os.ReadFile(tapefile)
	if err != nil {
		return "", err
	}
	tape := string(bytes)
	for k, v := range variables {
		marker := fmt.Sprintf("${%v}", strings.ToUpper(k))
		tape = strings.ReplaceAll(tape, marker, v)
	}
	return tape, nil
}

func generateGif(tape string) error {
	cmd := exec.Command("vhs")
	cmd.Stdin = strings.NewReader(tape)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("failed to generate gif. %v", err)
	}
	return nil
}

type generateCmd struct {
	tapefile string
	outpath  string
}

func (*generateCmd) Name() string { return "generate" }

func (*generateCmd) Synopsis() string { return "Generate gif" }

func (*generateCmd) Usage() string { return "generate -tape <file> -out <dir>\n" }

func (cmd *generateCmd) SetFlags(f *flag.FlagSet) {
	f.StringVar(&cmd.tapefile, "tape", "", "tape file path")
	f.StringVar(&cmd.outpath, "out", "", "output directory path")
}

func (cmd *generateCmd) Execute(_ context.Context, f *flag.FlagSet, args ...any) subcommands.ExitStatus {
	if err := cmd.run(); err != nil {
		log.Println(err)
		return subcommands.ExitFailure
	}
	return subcommands.ExitSuccess
}

func (cmd *generateCmd) run() error {
	if cmd.tapefile == "" {
		return errors.New("tape is not set")
	}
	if cmd.outpath == "" {
		return errors.New("out is not set")
	}
	if err := checkVhs(); err != nil {
		return err
	}
	variables := map[string]string{
		"output_dir": cmd.outpath,
	}
	tape, err := readTape(cmd.tapefile, variables)
	if err != nil {
		return err
	}
	if err := generateGif(tape); err != nil {
		return err
	}
	return nil
}

func main() {
	subcommands.Register(subcommands.HelpCommand(), "")
	subcommands.Register(subcommands.CommandsCommand(), "")
	subcommands.Register(subcommands.FlagsCommand(), "")
	subcommands.Register(&generateCmd{}, "")
	flag.Parse()
	ctx := context.Background()
	os.Exit(int(subcommands.Execute(ctx)))
}
