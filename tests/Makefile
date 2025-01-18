all: hello_world_c.elf sub_process_count_go.elf

# hello_world.c -> hello_world_c.elf
hello_world_c.elf: hello_world.c
	gcc -o hello_world_c.elf hello_world.c

# sub_process_count.go -> sub_process_count_go.elf
sub_process_count_go.elf: sub_process_count.go
	go build -o sub_process_count_go.elf sub_process_count.go

# Target to clean up generated files
clean:
	rm -f *.elf
