package main

import (
	"log/slog"
	"os"
	"os/exec"
	"path"
)

func main() {
	slog.Info("hello from brutus client")

	server_bin := os.Getenv("BRUTUS_SERVER_BIN")
	wd, _ := os.Getwd()

	cmd := exec.Command(path.Join(wd, server_bin))
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin
	cmd.Run()
	cmd.Wait()
}
