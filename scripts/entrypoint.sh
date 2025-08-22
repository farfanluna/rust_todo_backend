#!/bin/sh

set -e

# Asegurarse de que el directorio de la base de datos exista y tenga los permisos correctos
mkdir -p /app/database
chmod -R 777 /app/database

# Ejecutar migraciones, pasando explícitamente la URL de la base de datos
echo "Ejecutando migraciones de la base de datos..."
sqlx migrate run --source ./migrations --database-url "$DATABASE_URL"

# Iniciar la aplicación
echo "Iniciando la aplicación..."
exec ./rust_todo_backend
