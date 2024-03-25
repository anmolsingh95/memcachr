import os
import subprocess
import time

import pymemcache
import pytest


@pytest.fixture(scope="function")
def memcache_client():
    server_process = subprocess.Popen(["./target/debug/memcachr"])
    # Wait for the server to start
    time.sleep(0.5)
    client = pymemcache.client.base.Client(("127.0.0.1", 11212))
    log_info("start test")
    yield client
    log_info("end test")

    client.close()
    server_process.terminate()
    server_process.wait()


def test_set_and_get(memcache_client):
    log_info("set somekey")
    memcache_client.set("somekey", "somevalue")
    log_info("get somekey")
    assert b"somevalue" == memcache_client.get("somekey")
    log_info("get otherkey")
    assert None == memcache_client.get("otherkey")


def test_set_and_set_ttl(memcache_client):
    log_info("set somekey")
    memcache_client.set("somekey", "somevalue", expire=1)
    log_info("get somekey")
    assert b"somevalue" == memcache_client.get("somekey")
    log_info("wait 2 seconds")
    time.sleep(2)
    log_info("get somekey")
    assert None == memcache_client.get("somekey")


def log_info(msg):
    pid = os.getpid()
    unix_time_us = int(time.time() * 1e6)
    print(f"Testr: [{pid}] [{unix_time_us}]: {msg}")
