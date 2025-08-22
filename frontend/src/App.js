import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import axios from 'axios';
import './App.css';

// Componentes
import Login from './components/Login';
import Register from './components/Register';
import Dashboard from './components/Dashboard';
import AdminDashboard from './components/AdminDashboard';
import { useToast } from './hooks/use-toast';
import { Toaster } from './components/ui/toaster';
// Configurar axios con interceptors
axios.defaults.baseURL = '/api';
axios.interceptors.request.use(
  (config) => {
    const token = localStorage.getItem('token');
    if (token) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => Promise.reject(error)
);

axios.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response?.status === 401) {
      localStorage.removeItem('token');
      localStorage.removeItem('user');
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);

// Context para la autenticación
const AuthContext = React.createContext();

export const useAuth = () => {
  const context = React.useContext(AuthContext);
  if (!context) {
    throw new Error('useAuth debe usarse dentro de un AuthProvider');
  }
  return context;
};

const AuthProvider = ({ children }) => {
  const [user, setUser] = useState(null);
  const [loading, setLoading] = useState(true);
  const { toast } = useToast();

  useEffect(() => {
    const token = localStorage.getItem('token');
    const savedUser = localStorage.getItem('user');
    
    if (token && savedUser) {
      try {
        const userData = JSON.parse(savedUser);
        setUser(userData);
      } catch (error) {
        console.error('Error parsing saved user:', error);
        localStorage.removeItem('token');
        localStorage.removeItem('user');
      }
    }
    setLoading(false);
  }, []);

  const login = async (email, password) => {
    try {
      const response = await axios.post('/auth/login', { email, password });
      const { token, user: userData } = response.data;
      
      localStorage.setItem('token', token);
      localStorage.setItem('user', JSON.stringify(userData));
      setUser(userData);
      
      toast({
        title: "¡Bienvenido!",
        description: `Hola ${userData.name}, has iniciado sesión correctamente.`,
        variant: "default",
      });
      
      return { success: true };
    } catch (error) {
      const message = error.response?.data?.detail || 'Error al iniciar sesión';
      toast({
        title: "Error de autenticación",
        description: message,
        variant: "destructive",
      });
      return { success: false, error: message };
    }
  };

  const register = async (name, email, password) => {
    try {
      await axios.post('/auth/register', { name, email, password });
      
      toast({
        title: "Registro exitoso",
        description: "Tu cuenta ha sido creada. Ahora puedes iniciar sesión.",
        variant: "default",
      });
      
      return { success: true };
    } catch (error) {
      const message = error.response?.data?.error?.message || error.response?.data?.detail || 'Error al registrarse';
      
      // No mostrar toast si el error es por correo duplicado
      if (error.response?.status !== 409) {
        toast({
          title: "Error de registro",
          description: message,
          variant: "destructive",
        });
      }
      
      return { success: false, error: message };
    }
  };

  const logout = () => {
    localStorage.removeItem('token');
    localStorage.removeItem('user');
    setUser(null);
    toast({
      title: "Sesión cerrada",
      description: "Has cerrado sesión correctamente.",
      variant: "default",
    });
  };

  const value = {
    user,
    login,
    register,
    logout,
    isAuthenticated: !!user,
    isAdmin: user?.role === 'admin'
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-950 flex items-center justify-center">
        <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-500"></div>
      </div>
    );
  }

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  );
};

// Componente para proteger rutas
const ProtectedRoute = ({ children, adminOnly = false }) => {
  const { isAuthenticated, isAdmin } = useAuth();

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />;
  }

  if (adminOnly && !isAdmin) {
    return <Navigate to="/dashboard" replace />;
  }

  return children;
};

// Componente principal
function App() {
  return (
    <div className="App">
      <AuthProvider>
        <Router>
          <Routes>
            <Route 
              path="/" 
              element={<Navigate to="/dashboard" replace />} 
            />
            <Route 
              path="/login" 
              element={<Login />} 
            />
            <Route 
              path="/register" 
              element={<Register />} 
            />
            <Route
              path="/dashboard"
              element={
                <ProtectedRoute>
                  <Dashboard />
                </ProtectedRoute>
              }
            />
            <Route
              path="/admin"
              element={
                <ProtectedRoute adminOnly>
                  <AdminDashboard />
                </ProtectedRoute>
              }
            />
          </Routes>
        </Router>
        <Toaster />
      </AuthProvider>
    </div>
  );
}

export default App;
