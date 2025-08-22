set -e

echo "🚀 Configurando proyecto Rust Todo API..."

# Verificar que Rust esté instalado
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust no está instalado. Instálalo desde https://rustup.rs/"
    exit 1
fi

# Crear archivo .env si no existe
if [ ! -f .env ]; then
    echo "📝 Creando archivo .env..."
    cp .env.example .env
    echo "✅ Archivo .env creado. Por favor, revisa y ajusta las configuraciones."
fi

# Instalar sqlx-cli si no está instalado
if ! command -v sqlx &> /dev/null; then
    echo "📦 Instalando sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features sqlite
fi

# Navegar al directorio del backend
cd backend

# Ejecutar migraciones
echo "🗄️ Ejecutando migraciones de base de datos..."
sqlx migrate run

# Compilar el proyecto
echo "🔨 Compilando proyecto..."
cargo build

# Regresar al directorio raíz
cd ..

echo "✅ ¡Configuración completada!"
echo ""
echo "Para iniciar el servidor ejecuta:"
echo "  cargo run"
echo ""
echo "Una vez iniciado, visita:"
echo "  - API: http://127.0.0.1:3000"
echo "  - Swagger UI: http://127.0.0.1:3000/swagger-ui"
echo ""
echo "Usuario demo:"
echo "  - Email: lic.farfanluna@hotmail.com"
echo "  - Contraseña: demo123"
