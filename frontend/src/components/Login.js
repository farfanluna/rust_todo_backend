import React, { useState, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useAuth } from '../App';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Card, CardHeader, CardContent, CardTitle } from './ui/card';
import { Eye, EyeOff, Lock, Mail, LogIn } from 'lucide-react';

const Login = () => {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const { login, isAuthenticated } = useAuth();
  const navigate = useNavigate();
  const { t, i18n } = useTranslation();

  useEffect(() => {
    i18n.changeLanguage('en');
  }, [i18n]);

  // Redirigir si ya está autenticado
  React.useEffect(() => {
    if (isAuthenticated) {
      navigate('/dashboard');
    }
  }, [isAuthenticated, navigate]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setLoading(true);
    
    const result = await login(email, password);
    if (result.success) {
      navigate('/dashboard');
    }
    
    setLoading(false);
  };

  const handleDemoLogin = async (role) => {
    setLoading(true);
    if (role === 'admin') {
      const result = await login('admin@admin.com', 'admin123');
      if (result.success) {
        navigate('/dashboard');
      }
    }
    setLoading(false);
  };

  return (
    <div className="min-h-screen bg-gray-950 flex items-center justify-center p-4">
      {/* Fondo con gradiente sutil */}
      <div className="absolute inset-0 bg-gradient-to-br from-indigo-950/20 via-gray-950 to-purple-950/20"></div>
      
      <div className="relative z-10 w-full max-w-md">
        <div className="text-center mb-8">
          <div className="inline-flex items-center justify-center w-16 h-16 bg-indigo-600 rounded-full mb-4">
            <LogIn className="w-8 h-8 text-white" />
          </div>
          <h1 className="text-3xl font-bold text-white mb-2">TaskPro</h1>
          <p className="text-gray-400">{t('login.subtitle')}</p>
        </div>

        <Card className="bg-gray-900 border-gray-800 shadow-2xl">
          <CardHeader className="space-y-1">
            <CardTitle className="text-2xl text-center text-white">
              {t('login.title')}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="email" className="text-gray-300">
                  {t('login.email')}
                </Label>
                <div className="relative">
                  <Mail className="absolute left-3 top-3 h-4 w-4 text-gray-500" />
                  <Input
                    id="email"
                    type="email"
                    placeholder={t('login.emailPlaceholder')}
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    className="pl-10 bg-gray-800 border-gray-700 text-white placeholder-gray-500 focus:border-indigo-500 focus:ring-indigo-500"
                    required
                  />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="text-gray-300">
                  {t('login.password')}
                </Label>
                <div className="relative">
                  <Lock className="absolute left-3 top-3 h-4 w-4 text-gray-500" />
                  <Input
                    id="password"
                    type={showPassword ? 'text' : 'password'}
                    placeholder={t('login.passwordPlaceholder')}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    className="pl-10 pr-10 bg-gray-800 border-gray-700 text-white placeholder-gray-500 focus:border-indigo-500 focus:ring-indigo-500"
                    required
                  />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-3 text-gray-500 hover:text-gray-300"
                  >
                    {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  </button>
                </div>
              </div>

              <Button
                type="submit"
                className="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2.5"
                disabled={loading}
              >
                {loading ? (
                  <div className="flex items-center space-x-2">
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    <span>{t('login.loggingIn')}</span>
                  </div>
                ) : (
                  t('login.loginButton')
                )}
              </Button>
            </form>

            {/* Separador */}
            <div className="relative">
              <div className="absolute inset-0 flex items-center">
                <div className="w-full border-t border-gray-700" />
              </div>
              <div className="relative flex justify-center text-xs uppercase">
                <span className="bg-gray-900 px-2 text-gray-500">{t('login.orTryWith')}</span>
              </div>
            </div>

            {/* Demo buttons */}
            <div className="space-y-2">
              <Button
                type="button"
                onClick={() => handleDemoLogin('admin')}
                className="w-full bg-emerald-600 hover:bg-emerald-700 text-white font-medium py-2"
                disabled={loading}
              >
                {t('login.demoAdmin')}
              </Button>
            </div>

            <div className="text-center pt-4">
              <p className="text-gray-400 text-sm">
                {t('login.noAccount')}{' '}
                <Link 
                  to="/register" 
                  className="text-indigo-400 hover:text-indigo-300 font-medium transition-colors duration-200"
                >
                 {t('login.registerHere')}
                </Link>
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Login;
