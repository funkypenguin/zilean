package utils

import (
	"github.com/MunifTanjim/go-ptt"
	pb "github.com/ipromknight/zilean/src/GoPttServer/torrentparser"
)

func MapToProto(r *ptt.Result, infoHash string, originalTitle string) *pb.TorrentTitleResponse {
	return &pb.TorrentTitleResponse{
		InfoHash:      infoHash,
		OriginalTitle: originalTitle,
		Audio:         r.Audio,
		BitDepth:      r.BitDepth,
		Channels:      r.Channels,
		Codec:         r.Codec,
		Commentary:    r.Commentary,
		Complete:      r.Complete,
		Container:     r.Container,
		Convert:       r.Convert,
		Date:          r.Date,
		Documentary:   r.Documentary,
		Dubbed:        r.Dubbed,
		Edition:       r.Edition,
		EpisodeCode:   r.EpisodeCode,
		Episodes:      ToInt32Slice(r.Episodes),
		Extended:      r.Extended,
		Extension:     r.Extension,
		Group:         r.Group,
		Hdr:           r.HDR,
		Hardcoded:     r.Hardcoded,
		Languages:     r.Languages,
		Network:       r.Network,
		Proper:        r.Proper,
		Quality:       r.Quality,
		Region:        r.Region,
		Remastered:    r.Remastered,
		Repack:        r.Repack,
		Resolution:    r.Resolution,
		Retail:        r.Retail,
		Seasons:       ToInt32Slice(r.Seasons),
		Site:          r.Site,
		Size:          r.Size,
		Subbed:        r.Subbed,
		ThreeD:        r.ThreeD,
		Title:         r.Title,
		Uncensored:    r.Uncensored,
		Unrated:       r.Unrated,
		Upscaled:      r.Upscaled,
		Volumes:       ToInt32Slice(r.Volumes),
		Year:          r.Year,
	}
}
