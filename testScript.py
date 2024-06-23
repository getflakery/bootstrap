machine.start()
# assert /foo/bar.txt contains secret 
machine1.wait_for_file("/foo/bar.txt")
response = machine1.succeed("cat /foo/bar.txt")
assert "secret" in response

response = machine1.succeed("journalctl -xeu bootstrap.service --no-pager | grep -Eo run.+.service")
print(response)
assert ".service" in response
assert "run" in response

# wait for the service in response to finish by checking the status for
# ░░ The unit run-r7c73cc8979ef44e08e10d0d5f3713395.service completed and consumed the indicated resources.
response = machine1.wait_until_succeeds(f"journalctl -xeu {response} --no-pager | grep -Eo 'completed and consumed the indicated resources'", 30)
print(response)

# todo add me back
response = machine1.succeed("sqlite3 /tmp/db.sqlite3 'SELECT * FROM target;'")
print(response)
assert "00f00f" in response




response = machine1.succeed("sqlite3 /tmp/db.sqlite3 \"SELECT production FROM deployments WHERE id == '00f00f';\"")
print(response)
assert "1" == response