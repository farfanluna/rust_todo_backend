-- =================================================================
-- BORRADO DE TABLAS (PARA REINICIO LIMPIO)
-- =================================================================
DROP TABLE IF EXISTS login_attempts;
DROP TABLE IF EXISTS rate_limits;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS users;

-- =================================================================
-- ESTRUCTURA DE TABLAS
-- =================================================================

-- Tabla de usuarios, con la columna 'role' incluida desde el principio.
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    email TEXT UNIQUE NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    role TEXT CHECK(role IN ('user', 'admin')) NOT NULL DEFAULT 'user'
);

-- Tabla de tareas (sin cambios de estructura).
CREATE TABLE IF NOT EXISTS tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    status TEXT CHECK(status IN ('todo', 'doing', 'done')) DEFAULT 'todo',
    priority TEXT CHECK(priority IN ('low', 'med', 'high')) DEFAULT 'med',
    due_date TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    tags TEXT, -- Almacenado como string separado por comas
    assigned_to TEXT -- Nueva columna para la persona asignada
);

-- Nuevas Tablas de Seguridad.
CREATE TABLE IF NOT EXISTS rate_limits (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ip_address TEXT NOT NULL,
    endpoint TEXT NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 1,
    window_start TEXT NOT NULL,
    blocked_until TEXT,
    created_at TEXT DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS login_attempts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ip_address TEXT NOT NULL,
    email TEXT,
    success BOOLEAN NOT NULL DEFAULT FALSE,
    user_agent TEXT,
    attempted_at TEXT DEFAULT CURRENT_TIMESTAMP,
    blocked_until TEXT
);


-- =================================================================
-- ÍNDICES
-- =================================================================

-- Índices existentes
CREATE INDEX IF NOT EXISTS idx_tasks_user_id ON tasks(user_id);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
CREATE INDEX IF NOT EXISTS idx_tasks_due_date ON tasks(due_date);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Nuevos Índices de Seguridad
CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);
CREATE INDEX IF NOT EXISTS idx_rate_limits_ip_endpoint ON rate_limits(ip_address, endpoint);
CREATE INDEX IF NOT EXISTS idx_rate_limits_window_start ON rate_limits(window_start);
CREATE INDEX IF NOT EXISTS idx_login_attempts_ip ON login_attempts(ip_address);
CREATE INDEX IF NOT EXISTS idx_login_attempts_email ON login_attempts(email);
CREATE INDEX IF NOT EXISTS idx_login_attempts_attempted_at ON login_attempts(attempted_at);


-- =================================================================
-- DATOS DE DEMO COMPLETOS
-- =================================================================

-- -----------------------------------------------------------------
-- Usuario 1: Jesús Farfán Luna (contraseña: "demo123"), Rol: user
-- -----------------------------------------------------------------
INSERT OR IGNORE INTO users (id, name, email, password_hash, role) VALUES 
(1, 'Jesús Farfán Luna', 'lic.farfanluna@hotmail.com', '$2a$12$nK/W51KxJWoJ3Ms7FVGMTuiEzsonJ8Mx33QTh55PN/SztO41gNT5y', 'user');

-- Tareas para Jesús Farfán Luna (Lista completa)
INSERT OR IGNORE INTO tasks (id, user_id, title, description, status, priority, due_date, tags, assigned_to) VALUES 
(1, 1, 'Configurar proyecto Rust', 'Inicializar el proyecto con Cargo y configurar dependencias.', 'done', 'high', '2025-08-15 09:00:00', 'rust,setup,backend', 'Jesús Farfán Luna'),
(2, 1, 'Implementar autenticación JWT', 'Crear sistema de login y registro con tokens JWT.', 'doing', 'high', '2025-08-20 17:00:00', 'auth,jwt,security', 'Jesús Farfán Luna'),
(3, 1, 'Crear endpoints de tareas (CRUD)', 'Desarrollar CRUD completo para gestión de tareas.', 'todo', 'med', '2025-08-22 12:00:00', 'api,crud,tasks', 'Admin User'),
(4, 1, 'Agregar filtros y búsqueda', 'Implementar filtros avanzados para status, priority y búsqueda de texto.', 'todo', 'med', '2025-08-25 15:00:00', 'filters,search,api', 'Admin User'),
(5, 1, 'Documentar API', 'Crear documentación interactiva con Swagger/OpenAPI usando Utoipa.', 'doing', 'low', '2025-08-30 18:00:00', 'docs,swagger,api', 'Jesús Farfán Luna'),
(6, 1, 'Escribir pruebas unitarias', 'Implementar pruebas para los handlers y la lógica de negocio.', 'todo', 'low', '2025-09-01 10:00:00', 'testing,unit,quality', 'Super Admin'),
(7, 1, 'Revisar el código y refactorizar', NULL, 'todo', 'low', '2025-09-05 11:00:00', 'quality,refactor', 'Super Admin'),
(8, 1, 'Preparar la presentación del proyecto', 'Crear diapositivas para la demo final.', 'todo', 'med', NULL, 'presentation,demo', 'Jesús Farfán Luna'),
(9, 1, 'Desplegar la aplicación', 'Configurar el entorno de producción y desplegar la API.', 'todo', 'high', '2025-09-10 23:59:59', 'deploy,production,server', NULL),
(10, 1, 'Arreglar bug en la paginación', 'El conteo total de páginas es incorrecto cuando no hay filtros.', 'doing', 'high', '2025-08-19 18:00:00', 'bug,api,pagination', 'Jesús Farfán Luna'),
(11, 1, 'Comprar dominio para el proyecto', NULL, 'done', 'low', '2025-08-10 10:00:00', 'infra,production', NULL),
(12, 1, 'Investigar sobre WebSockets para notificaciones', 'Evaluar la posibilidad de añadir notificaciones en tiempo real.', 'todo', 'low', NULL, 'research,websockets,notifications', NULL);

-- -----------------------------------------------------------------
-- Usuario 2: Admin User (contraseña: "admin123"), Rol: admin
-- -----------------------------------------------------------------
INSERT OR IGNORE INTO users (id, name, email, password_hash, role) VALUES 
(2, 'Admin User', 'admin@admin.com', '$2a$12$g6M.2QSNiMj1AQCBAi70e.VccJdQFGRTY6/UuTbDI2mRSqcPp4MNa', 'admin');

-- Tareas para Admin User (Lista completa)
INSERT OR IGNORE INTO tasks (id, user_id, title, description, status, priority, due_date, tags, assigned_to) VALUES 
(13, 2, 'Revisar logs del servidor', 'Analizar los logs en busca de errores o actividad sospechosa.', 'doing', 'high', '2025-08-20 10:00:00', 'server,admin,security', 'Admin User'),
(14, 2, 'Hacer backup de la base de datos', 'Realizar una copia de seguridad completa de la base de datos de producción.', 'done', 'med', '2025-08-18 03:00:00', 'db,admin,backup', 'Admin User'),
(15, 2, 'Actualizar dependencias del proyecto', 'Revisar y actualizar las versiones de las librerías a las últimas estables.', 'todo', 'low', '2025-09-02 12:00:00', 'maintenance,rust', NULL),
(16, 2, 'Monitorear el rendimiento de la API', 'Configurar dashboards para visualizar la latencia y el uso de CPU.', 'todo', 'med', NULL, 'monitoring,performance,server', 'Super Admin');

-- -----------------------------------------------------------------
-- Nuevo Usuario 3: Super Admin (contraseña: superadmin), Rol: admin
-- -----------------------------------------------------------------
INSERT OR IGNORE INTO users (id, name, email, password_hash, role) VALUES 
(3, 'Super Admin', 'superadmin@admin.com', '$2a$12$g6M.2QSNiMj1AQCBAi70e.VccJdQFGRTY6/UuTbDI2mRSqcPp4MNa', 'admin');
