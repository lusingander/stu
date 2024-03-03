package main

import (
	"bytes"
	"fmt"
	"log"
	"os"
	"os/exec"
	"strings"
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

func generateGif() error {
	cmd := exec.Command("vhs", "./tool/gifgen/tape/default.tape")
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	if err := cmd.Run(); err != nil {
		return fmt.Errorf("Failed to generate gif. %v", err)
	}
	return nil
}

func run() error {
	if err := checkVhs(); err != nil {
		return err
	}
	if err := generateGif(); err != nil {
		return err
	}
	return nil
}

func main() {
	if err := run(); err != nil {
		log.Fatal(err)
	}
}
