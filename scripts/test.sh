set -e

echo "🧪 Ejecutando pruebas..."

# Navegar al directorio del backend
cd backend

# Ejecutar tests con logs
cargo test -- --nocapture

echo "✅ Todas las pruebas pasaron!"
