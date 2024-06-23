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

# Strip any extra whitespace from the response
response = response.strip()
# remove '' from the response
response = response[1:-1]


# Wait for the service in the response to complete
status_check_command = f"journalctl -xeu --no-pager {response} | grep -Eo 'completed and consumed the indicated resources'"
response = machine1.wait_until_succeeds(status_check_command, 300)
print(response)


# todo add me back
response = machine1.succeed("sqlite3 /tmp/db.sqlite3 'SELECT * FROM target;'")
print(response)
assert "00f00f" in response




response = machine1.succeed("sqlite3 /tmp/db.sqlite3 \"SELECT production FROM deployments WHERE id == '00f00f';\"")
print(response)
assert "1" == response