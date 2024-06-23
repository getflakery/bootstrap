machine.start()
# assert /foo/bar.txt contains secret 
machine1.wait_for_file("/foo/bar.txt")
response = machine1.succeed("cat /foo/bar.txt")
assert "secret" in response

# Run the command to get the service name from the journal logs
response = machine1.succeed("journalctl -xeu bootstrap.service --no-pager | grep -Eo 'run-.+\\.service'")
print(response)

# Ensure the response contains a service name
assert ".service" in response
assert "run" in response

machine1.wait_for_file("/tmp/rebuilt")

# todo add me back
response = machine1.succeed("sqlite3 /tmp/db.sqlite3 'SELECT * FROM target;'")
print(response)
assert "00f00f" in response




response = machine1.succeed("sqlite3 /tmp/db.sqlite3 \"SELECT production FROM deployments WHERE id == '00f00f';\"")
print(response)
assert "1" == response