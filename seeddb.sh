#!/bin/sh

# Stop on any error
set -e

# Create tables
sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS templates (id TEXT PRIMARY KEY);"
sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS files (id TEXT PRIMARY KEY, path TEXT NOT NULL, content TEXT NOT NULL, user_id TEXT NOT NULL, initialization_vector TEXT NOT NULL);"
sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS template_files (id TEXT PRIMARY KEY, file_id TEXT NOT NULL, template_id TEXT NOT NULL);"

# // instanceDeployment
# export const deployments = sqliteTable('deployments', {
#   id: text('id').primaryKey(),
#   name: text('name').notNull(),
#   templateID: text('template_id').notNull().references(() => templates.id, { onDelete: 'no action' }).notNull(),
#   userID: text('user_id').notNull(),
#   awsInstanceID: text('aws_instance_id'),
#   createdAt: integer('created_at').notNull(),
#   host: text('host'),
#   port: integer('port'),
#   data: text('data', { mode: 'json' }).$type<{ 
#     port_mappings: {
#       lb_port: number,
#       instance_port: number,
#     }[],
#     aws_resources: { 
#       security_group_id: string,
#       launch_template_id: string,
#       autoscaling_group_id: string,
#     },
#    }>(),
# });

# export const target = sqliteTable('target', {
#   id: text('id').primaryKey(),
#   deploymentID: text('deployment_id').notNull().references(() => deployments.id, { onDelete: 'cascade' }).notNull(),
#   host: text('host').notNull(),
# });
sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS deployments (id TEXT PRIMARY KEY, name TEXT NOT NULL, template_id TEXT NOT NULL, user_id TEXT NOT NULL, aws_instance_id TEXT, created_at INTEGER NOT NULL, host TEXT, port INTEGER, data TEXT NOT NULL);"
sqlite3 /tmp/db.sqlite3 "CREATE TABLE IF NOT EXISTS target (id TEXT PRIMARY KEY, deployment_id TEXT NOT NULL, host TEXT NOT NULL);"

# Insert data
sqlite3 /tmp/db.sqlite3 "INSERT INTO templates (id) VALUES ('0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f');"
sqlite3 /tmp/db.sqlite3 "INSERT INTO files (id, path, content, user_id, initialization_vector) VALUES ('474dc715fcef9838628de248b91ad845', '/foo/bar.txt', '474dc715fcef9838628de248b91ad845', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '391827ead4c1a7fdad2dd9256d01a57a');"
sqlite3 /tmp/db.sqlite3 "INSERT INTO template_files (id, file_id, template_id) VALUES ('0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '474dc715fcef9838628de248b91ad845', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f');"
# create deployment with id 00f00f
sqlite3 /tmp/db.sqlite3 "INSERT INTO deployments (id, name, template_id, user_id, created_at, data) VALUES ('00f00f', 'deployment1', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', '0939865eee0fff95518bb8f0ac64cafe5d9d04429b51d55a82d3a42ea5da5b1f', 123456789, '{\"port_mappings\":[{\"lb_port\":80,\"instance_port\":80}],\"aws_resources\":{\"security_group_id\":\"sg-123456\",\"launch_template_id\":\"lt-123456\",\"autoscaling_group_id\":\"asg-123456\"}');"