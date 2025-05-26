package grpc_server

import (
	"google.golang.org/grpc"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"

	pb "github.com/ipromknight/zilean/src/GoPttServer/torrentparser"
)

type Server struct {
	pb.UnimplementedTorrentParserServer
	workerCount int
	stopFunc    func()
}

func New(workerCount int) *Server {
	return &Server{workerCount: workerCount}
}

func (s *Server) Start(socketPath string) {
	removeSocket(socketPath)

	lis, err := net.Listen("unix", socketPath)
	if err != nil {
		log.Fatalf("Failed to listen: %v", err)
	}
	defer func() {
		removeSocket(socketPath)
		log.Println("Listener closed and socket removed")
	}()

	if err := os.Chmod(socketPath, 0770); err != nil {
		log.Fatalf("Failed to chmod socket: %v", err)
	}

	grpcServer := grpc.NewServer()
	pb.RegisterTorrentParserServer(grpcServer, s)

	s.stopFunc = func() {
		log.Println("Shutting down gRPC server...")
		grpcServer.GracefulStop()
	}

	go func() {
		sig := make(chan os.Signal, 1)
		signal.Notify(sig, syscall.SIGINT, syscall.SIGTERM)
		<-sig
		s.stopFunc()
	}()

	log.Printf("gRPC server listening at: %s", socketPath)
	if err := grpcServer.Serve(lis); err != nil {
		log.Fatalf("gRPC Serve error: %v", err)
	}
}

func removeSocket(socketPath string) {
	if _, err := os.Stat(socketPath); err == nil {
		log.Printf("Socket file %s exists, removing it", socketPath)
		_ = os.Remove(socketPath)
	}
}
