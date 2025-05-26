package grpc_server

import (
	pb "github.com/ipromknight/zilean/src/GoPttServer/torrentparser"
)

type job struct {
	req  *pb.TorrentTitleRequest
	resp chan *pb.TorrentTitleResponse
}
