# API de To-Do en Rust 🦀

Una API RESTful completa para gestión de tareas desarrollada en Rust con Axum, SQLx y autenticación JWT.

## ✨ Características

- **🔐 Autenticación JWT** - Sistema completo de registro, login y autenticación
- **📝 CRUD de Tareas** - Crear, leer, actualizar y eliminar tareas
- **🔍 Búsqueda Avanzada** - Filtros por estado, prioridad, tags, fechas y búsqueda de texto
- **📄 Paginación** - Navegación eficiente con metadatos de paginación
- **📚 Documentación Swagger** - API totalmente documentada con OpenAPI
- **🛡️ Seguridad** - Validación de entrada, hashing de contraseñas con bcrypt
- **⚡ Rendimiento** - Base de datos SQLite con índices optimizados
- **🧪 Pruebas** - Suite de pruebas de integración incluidas
- **🐳 Dockerizado** - Listo para producción con `docker-compose`

## 🚀 Inicio Rápido (Recomendado con Docker)

### Prerrequisitos

- Docker y Docker Compose

### Ejecución

1. **Clonar el repositorio**
   ```bash
   git clone <repo-url>
   cd rust_todo_backend
   ```

2. **Configurar variables de entorno**
   Copia el archivo de ejemplo y ajústalo si es necesario.
   ```bash
   cp .env.example .env
   ```

3. **Levantar los servicios**
   Este comando construirá la imagen de Rust y levantará el servidor.
   ```bash
   docker-compose up --build
   ```

El servidor estará disponible en:
- **Frontend**: `http://127.0.0.1`
- **API**: `http://127.0.0.1/api`
- **Swagger UI**: `http://127.0.0.1/swagger-ui`

Para detener los servicios, presiona `Ctrl+C` y luego ejecuta:
```bash
docker-compose down
```

---

## 🛠️ Instalación Manual

### Prerrequisitos

- Rust 1.70+ (instalar desde [rustup.rs](https://rustup.rs/))
- SQLite (incluido en la mayoría de sistemas)
- `sqlx-cli` para las migraciones

### Instalación

1. **Clonar el repositorio**
   ```bash
   git clone <repo-url>
   cd rust_todo_backend
   ```

2. **Configurar variables de entorno**
   ```bash
   cp .env.example .env
   # Editar .env con tu configuración
   ```

3. **Instalar dependencias y compilar**
   ```bash
   cargo build
   ```

4. **Ejecutar migraciones**
   ```bash
   cargo install sqlx-cli --no-default-features --features sqlite
   sqlx migrate run
   ```

5. **Iniciar el servidor**
   ```bash
   cargo run
   ```

El servidor estará disponible en:
- **API**: `http://127.0.0.1:3000`
- **Swagger UI**: `http://127.0.0.1:3000/swagger-ui`

## 🔧 Configuración

### Variables de Entorno (.env)

```bash
# Base de datos
DATABASE_URL=sqlite:todo.db

# JWT
JWT_SECRET=tu_clave_secreta_jwt_de_al_menos_32_caracteres
JWT_EXPIRATION_HOURS=24

# Servidor
PORT=3000
HOST=127.0.0.1

# Logging
RUST_LOG=debug
```

### Usuario de Demostración

La migración incluye un usuario demo:
- **Email**: `demo@todoapi.com`
- **Contraseña**: `demo123`

## 📖 Uso de la API

### Autenticación

#### Registrar Usuario
```bash
curl -X POST http://localhost:3000/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Juan Pérez",
    "email": "juan@example.com",
    "password": "mi_password_seguro"
  }'
```

#### Iniciar Sesión
```bash
curl -X POST http://localhost:3000/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "juan@example.com",
    "password": "mi_password_seguro"
  }'
```

Respuesta:
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": 1,
    "name": "Juan Pérez",
    "email": "juan@example.com",
    "created_at": "2025-08-18T10:00:00Z"
  }
}
```

#### Obtener Usuario Actual
```bash
curl -X GET http://localhost:3000/me \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### Gestión de Tareas

#### Crear Tarea
```bash
curl -X POST http://localhost:3000/tasks \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Implementar nueva funcionalidad",
    "description": "Desarrollar el módulo de reportes",
    "status": "todo",
    "priority": "high",
    "due_date": "2025-08-25T17:00:00Z",
    "tags": "desarrollo,backend,rust"
  }'
```

#### Obtener Tareas con Filtros
```bash
# Tareas básicas
curl -X GET "http://localhost:3000/tasks" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"

# Con filtros avanzados
curl -X GET "http://localhost:3000/tasks?status=todo,doing&priority=high&search=API&sort_by=due_date&sort_order=asc&page=1&per_page=5" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

#### Actualizar Tarea
```bash
curl -X PUT http://localhost:3000/tasks/1 \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "status": "doing",
    "priority": "med"
  }'
```

#### Eliminar Tarea
```bash
curl -X DELETE http://localhost:3000/tasks/1 \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

## 🔍 Filtros y Búsqueda

La API soporta filtros avanzados en `GET /tasks`:

| Parámetro | Descripción | Ejemplo |
|-----------|-------------|---------|
| `status` | Estados (separados por coma) | `todo,doing,done` |
| `priority` | Prioridades (separadas por coma) | `high,med,low` |
| `tags` | Búsqueda en tags | `rust,api` |
| `search` | Búsqueda en título/descripción | `documentación` |
| `sort_by` | Campo de ordenación | `created_at,due_date,title,priority,status` |
| `sort_order` | Orden | `asc,desc` |
| `page` | Página (empezando en 1) | `1` |
| `per_page` | Elementos por página (máx 100) | `10` |
| `due_date_start` | Fecha inicio | `2025-08-01T00:00:00Z` |
| `due_date_end` | Fecha fin | `2025-12-31T23:59:59Z` |

## 🏗️ Arquitectura

```
rust_todo_backend/
├── Cargo.toml                 ✅ Dependencias completas
├── .env.example              ✅ Variables de entorno template
├── .gitignore                ✅ Archivos a ignorar
├── README.md                 ✅ Documentación completa
├── docker-compose.yml        ✅ Para deployment
├── migrations/
│   └── 0001_init.sql         ✅ Schema con datos demo
├── scripts/
│   ├── setup.sh              ✅ Setup automático
│   ├── test.sh               ✅ Ejecutar pruebas
│   └── dev.sh                ✅ Modo desarrollo
└── src/
    ├── Dockerfile            ✅ Para containerización (NUEVO)
    ├── main.rs               ✅ Punto de entrada
    ├── config.rs             ✅ Configuración
    ├── error.rs              ✅ Manejo de errores
    ├── db.rs                 ✅ Base de datos
    ├── models.rs             ✅ Modelos completos
    ├── routes.rs             ✅ Todos los endpoints
    ├── auth/
    │   ├── mod.rs            ✅ Módulo auth
    │   ├── jwt.rs            ✅ Gestión JWT
    │   └── middleware.rs     ✅ Middleware auth
    └── tests/
        ├── mod.rs            ✅ Módulo pruebas
        └── integration.rs    ✅ Pruebas integración
```

## 🧪 Pruebas

```bash
# Ejecutar todas las pruebas
cargo test

# Ejecutar pruebas con logs
cargo test -- --nocapture

# Ejecutar pruebas específicas
cargo test integration_tests
```

## 🛠️ Desarrollo

### Comandos Útiles

```bash
# Verificar código
cargo check

# Formatear código
cargo fmt

# Linter
cargo clippy

# Documentación
cargo doc --open

# Migraciones
sqlx migrate add <nombre_migracion>
sqlx migrate run
sqlx migrate revert
```

### Estructura de Base de Datos

#### Tabla `users`
- `id`: PRIMARY KEY
- `name`: Nombre del usuario
- `email`: Email único
- `password_hash`: Hash bcrypt de la contraseña
- `created_at`: Timestamp de creación

#### Tabla `tasks`
- `id`: PRIMARY KEY
- `user_id`: FK a users
- `title`: Título de la tarea
- `description`: Descripción opcional
- `status`: `todo`, `doing`, `done`
- `priority`: `low`, `med`, `high`
- `due_date`: Fecha límite (ISO 8601)
- `created_at`, `updated_at`: Timestamps
- `tags`: String separado por comas

## 📚 Documentación API

Una vez ejecutando el servidor, visita:
- **Swagger UI**: `http://localhost:3000/swagger-ui`
- **OpenAPI JSON**: `http://localhost:3000/api-docs/openapi.json`

## 🖥️ Aplicación de Escritorio (Tauri)

Este proyecto también se puede compilar como una aplicación de escritorio para Windows, macOS y Linux.

### Prerrequisitos

- Seguir los [prerrequisitos de Tauri](https://tauri.app/v1/guides/getting-started/prerequisites) para tu sistema operativo.

### Ejecución en Modo Desarrollo

```bash
# Navega a la carpeta del frontend
cd frontend

# Inicia la aplicación de escritorio
npm run tauri dev
```

### Compilación para Producción

```bash
# Navega a la carpeta del frontend
cd frontend/frontend

# Compila la aplicación
npm run tauri build
```

Los ejecutables se encontrarán en `frontend/frontend/src-tauri/target/release/`.

## 🤝 Contribución

1. Fork el proyecto
2. Crea una rama para tu funcionalidad (`git checkout -b feature/AmazingFeature`)
3. Commit tus cambios (`git commit -m 'Add some AmazingFeature'`)
4. Push a la rama (`git push origin feature/AmazingFeature`)
5. Abre un Pull Request

## 📄 Licencia

Este proyecto está bajo la Licencia MIT. Ver `LICENSE` para más detalles.

## 🆘 Soporte

Si tienes problemas o preguntas:

1. Revisa la [documentación de Swagger UI](http://localhost:3000/swagger-ui)
2. Consulta los logs con `RUST_LOG=debug cargo run`
3. Verifica que todas las variables de entorno estén configuradas
4. Asegúrate de que las migraciones se hayan ejecutado correctamente

---

### Generar Ejecutable

Para generar el ejecutable de la aplicación de escritorio, sigue estos pasos:

1. **Limpiar compilaciones anteriores y generar el ejecutable**
   ```bash
   rd /s /q frontend/src-tauri/target && cd frontend && npm run tauri build
   ```

2. **Ejecutar la aplicación**
   Una vez compilado, el ejecutable se encontrará en la siguiente ruta:
   ```bash
   \rust_todo_backend\frontend\src-tauri\target\release
   ```
   Puedes ejecutar el archivo `rust-todo-desktop.exe` desde esa carpeta.

   *Nota: La compilación requiere que `cargo` esté disponible en el PATH del sistema.*

Desarrollado con Rust
