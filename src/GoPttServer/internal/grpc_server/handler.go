package grpc_server

import (
	"context"
	"github.com/MunifTanjim/go-ptt"
	"io"
	"log"
	"sync"

	"github.com/ipromknight/zilean/src/GoPttServer/internal/utils"
	pb "github.com/ipromknight/zilean/src/GoPttServer/torrentparser"
)

func (s *Server) ParseTitles(stream pb.TorrentParser_ParseTitlesServer) error {
	jobs := make(chan job, 1000)
	var wg sync.WaitGroup

	for i := 0; i < s.workerCount; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for j := range jobs {
				parsed := ptt.Parse(j.req.Title)
				j.resp <- utils.MapToProto(parsed, j.req.InfoHash, j.req.Title)
			}
		}()
	}

	respChan := make(chan *pb.TorrentTitleResponse, 1000)
	done := make(chan struct{})

	go func() {
		for resp := range respChan {
			if err := stream.Send(resp); err != nil {
				log.Printf("Send error: %v", err)
			}
		}
		close(done)
	}()

	for {
		req, err := stream.Recv()
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Printf("Recv error: %v", err)
			break
		}
		jobs <- job{req: req, resp: respChan}
	}

	close(jobs)
	wg.Wait()
	close(respChan)
	<-done

	return nil
}

func (s *Server) Shutdown(ctx context.Context, req *pb.ShutdownRequest) (*pb.ShutdownResponse, error) {
	log.Println("Received shutdown signal via RPC")
	go s.stopFunc()
	return &pb.ShutdownResponse{}, nil
}
