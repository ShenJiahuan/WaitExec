# WaitExec
A distributed manager to schedule dependent jobs.

## Required

- MySQL

## Usage
Instantly run a program:
```shell
./wait_exec --config /path/to/your/wait_exec.toml 
--program 'echo "hello world"'
--instant
```
Start until a dependent job has finished:
```shell
./wait_exec --config /path/to/your/wait_exec.toml 
--program 'echo "hello world"'
--host 'some-machine'
--pid 2333
```