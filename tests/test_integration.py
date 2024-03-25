import subprocess
import time

import pymemcache
import pytest


@pytest.fixture(scope="function")
def memcache_client():
    server_process = subprocess.Popen(["./target/debug/memcachr"])
    # Wait for the server to start
    time.sleep(1)
    client = pymemcache.client.base.Client(("127.0.0.1", 11212))
    yield client

    client.close()
    server_process.terminate()
    server_process.wait()


def test_set_and_get(memcache_client):
    memcache_client.set("somekey", "somevalue")
    time.sleep(0.1)
    assert b"somevalue" == memcache_client.get("somekey")
    time.sleep(0.1)
    assert None == memcache_client.get("otherkey")


def test_set_and_set_ttl(memcache_client):
    memcache_client.set("somekey", "somevalue", expire=1)
    time.sleep(0.1)
    assert b"somevalue" == memcache_client.get("somekey")
    time.sleep(2)
    assert None == memcache_client.get("somekey")
