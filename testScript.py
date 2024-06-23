machine.start()
# assert /foo/bar.txt contains secret 
machine1.wait_for_file("/foo/bar.txt")
response = machine1.succeed("cat /foo/bar.txt")
assert "secret" in response
# todo add me back
response = machine1.succeed("sqlite3 /tmp/db.sqlite3 'SELECT * FROM target;'")
print(response)
assert "00f00f" in response
# select production from deployment
# response = machine1.succeed("sqlite3 /tmp/db.sqlite3 'SELECT production FROM deployments WHERE id == '00f00f';'")
response = machine1.succeed("sqlite3 /tmp/db.sqlite3 \"SELECT production FROM deployments WHERE id == '00f00f';\"")
print(response)
assert "1" == response