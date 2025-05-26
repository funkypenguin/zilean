import asyncio
import grpc
import logging
import os
import signal
import sys
from pathlib import Path

import torrent_parser_pb2
import torrent_parser_pb2_grpc
from RTN import parse as parse_title

SOCKET_PATH = os.getenv("ZILEAN_PTT_GRPC_SOCKET", "/tmp/zilean-torrent-parser.sock")
WORKER_COUNT = int(os.getenv("ZILEAN_PTT_WORKER_COUNT", "4"))

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
logger = logging.getLogger("PythonPTT")


class TorrentParserService(torrent_parser_pb2_grpc.TorrentParserServicer):
    def __init__(self, stop_event: asyncio.Event):
        self.semaphore = asyncio.Semaphore(WORKER_COUNT)
        self.stop_event = stop_event

    async def ParseTitles(self, request_iterator, context):
        async for request in request_iterator:
            async with self.semaphore:
                try:
                    result = parse_title(request.title)
                    logger.info(f"Parsed: {result.parsed_title}")
                    yield torrent_parser_pb2.TorrentTitleResponse(
                        info_hash=request.info_hash,
                        original_title=request.title,
                        title=result.parsed_title,
                        year=str(result.year or ""),
                        audio=result.audio,
                        bit_depth=result.bit_depth or "",
                        channels=result.channels,
                        codec=result.codec,
                        complete=result.complete,
                        container=result.container,
                        date=result.date,
                        documentary=result.documentary,
                        dubbed=result.dubbed,
                        edition=result.edition,
                        episode_code=result.episode_code or "",
                        episodes=result.episodes,
                        extended=result.extended,
                        extension=result.extension,
                        group=result.group,
                        hdr=result.hdr,
                        hardcoded=result.hardcoded,
                        languages=result.languages,
                        network=result.network,
                        proper=result.proper,
                        quality=result.quality,
                        region=result.region,
                        remastered=result.remastered,
                        repack=result.repack,
                        resolution=result.resolution,
                        retail=result.retail,
                        seasons=result.seasons,
                        site=result.site,
                        size=result.size or "",
                        subbed=result.subbed,
                        unrated=result.unrated,
                        upscaled=result.upscaled,
                        volumes=result.volumes,
                    )
                except Exception as e:
                    logger.error(f"Failed to parse {request.title}: {e}")

    async def Shutdown(self, request, context):
        logger.warning("Shutdown requested via gRPC")
        self.stop_event.set()
        return torrent_parser_pb2.ShutdownResponse()


async def serve():
    if Path(SOCKET_PATH).exists():
        Path(SOCKET_PATH).unlink()

    stop_event = asyncio.Event()

    server = grpc.aio.server()
    torrent_parser_pb2_grpc.add_TorrentParserServicer_to_server(TorrentParserService(stop_event), server)
    server.add_insecure_port(f'unix:{SOCKET_PATH}')
    await server.start()

    logger.info(f"Python gRPC server started at {SOCKET_PATH}")

    def handle_signal(*_):
        stop_event.set()

    loop = asyncio.get_running_loop()
    loop.add_signal_handler(signal.SIGTERM, handle_signal)
    loop.add_signal_handler(signal.SIGINT, handle_signal)

    await stop_event.wait()
    await server.stop(0)
    logger.info("Python gRPC server stopped")


if __name__ == "__main__":
    if sys.platform.startswith("windows"):
        logger.error("This service requires a Unix-based OS for Unix domain sockets")
        sys.exit(1)
    asyncio.run(serve())