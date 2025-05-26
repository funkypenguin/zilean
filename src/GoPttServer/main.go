package main

import (
	"github.com/ipromknight/zilean/src/GoPttServer/internal/grpc_server"
	"github.com/ipromknight/zilean/src/GoPttServer/internal/utils"
)

const (
	GRPC_SOCKET  = "ZILEAN_PTT_GRPC_SOCKET"
	WORKER_COUNT = "ZILEAN_PTT_WORKER_COUNT"
)

var (
	defaultSocket      = "/tmp/zilean-torrent-parser.sock"
	defaultWorkerCount = 4
)

func main() {
	socket := utils.GetEnv(GRPC_SOCKET, defaultSocket)
	workerCount := utils.GetEnvInt(WORKER_COUNT, defaultWorkerCount)

	s := grpc_server.New(workerCount)
	s.Start(socket)
}
