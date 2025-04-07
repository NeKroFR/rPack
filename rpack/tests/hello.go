package main

import (
	"fmt"
	"os"
	"os/exec"
	"strconv"
)

func main() {
	for i := 1; i <= 10; i++ {
		cmd := exec.Command(os.Args[0], strconv.Itoa(i))
		cmd.Stdout = os.Stdout
		cmd.Stderr = os.Stderr
		err := cmd.Run()
		if err != nil {
			fmt.Println("Error:", err)
		}
	}
}

func init() {
	if len(os.Args) > 1 {
		num, err := strconv.Atoi(os.Args[1])
		if err == nil {
			fmt.Println(num)
			os.Exit(0)
		}
	}
}
