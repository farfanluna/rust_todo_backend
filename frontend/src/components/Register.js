import React, { useState, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useAuth } from '../App';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Card, CardHeader, CardContent, CardTitle } from './ui/card';
import { Eye, EyeOff, Lock, Mail, User, UserPlus } from 'lucide-react';

const Register = () => {
  const [formData, setFormData] = useState({
    name: '',
    email: '',
    password: '',
    confirmPassword: ''
  });
  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);
  const [loading, setLoading] = useState(false);
  const [errors, setErrors] = useState({});
  
  const { register, isAuthenticated } = useAuth();
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

  const handleChange = (e) => {
    const { name, value } = e.target;
    setFormData(prev => ({
      ...prev,
      [name]: value
    }));
    
    // Limpiar errores cuando el usuario empieza a escribir
    if (errors[name]) {
      setErrors(prev => {
        const newErrors = { ...prev };
        delete newErrors[name];
        return newErrors;
      });
    }
  };

  const validateForm = (data) => {
    const newErrors = {};

    if (!data.name.trim()) {
      newErrors.name = t('register.nameRequired');
    } else if (data.name.trim().length < 2) {
      newErrors.name = t('register.nameMinLength', { length: 2 });
    }

    if (!data.email.trim()) {
      newErrors.email = t('register.emailRequired');
    } else if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(data.email)) {
      newErrors.email = t('register.emailInvalid');
    }

    if (!data.password) {
      newErrors.password = t('register.passwordRequired');
    } else if (data.password.length < 8) {
      newErrors.password = t('register.passwordMinLength', { length: 8 });
    }

    if (!data.confirmPassword) {
      newErrors.confirmPassword = t('register.confirmPasswordRequired');
    } else if (data.password !== data.confirmPassword) {
      newErrors.confirmPassword = t('register.passwordsNoMatch');
    }

    return newErrors;
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    
    const validationErrors = validateForm(formData);
    setErrors(validationErrors);

    if (Object.keys(validationErrors).length > 0) {
      return;
    }

    setLoading(true);
    
    const result = await register(formData.name, formData.email, formData.password);
    if (result.success) {
      navigate('/login');
    } else if (result.error) {
      // Si el error es sobre el correo, lo mostramos en el campo de email
      if (result.error.toLowerCase().includes('el email ya está registrado')) {
        setErrors({ email: t('register.emailInUse') });
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
            <UserPlus className="w-8 h-8 text-white" />
          </div>
          <h1 className="text-3xl font-bold text-white mb-2">TaskPro</h1>
          <p className="text-gray-400">{t('register.subtitle')}</p>
        </div>

        <Card className="bg-gray-900 border-gray-800 shadow-2xl">
          <CardHeader className="space-y-1">
            <CardTitle className="text-2xl text-center text-white">
              {t('register.createAccount')}
            </CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name" className="text-gray-300">
                  {t('register.fullName')}
                </Label>
                <div className="relative">
                  <User className="absolute left-3 top-3 h-4 w-4 text-gray-500" />
                  <Input
                    id="name"
                    name="name"
                    type="text"
                    placeholder={t('register.fullNamePlaceholder')}
                    value={formData.name}
                    onChange={handleChange}
                    className={`pl-10 bg-gray-800 border-gray-700 text-white placeholder-gray-500 focus:border-indigo-500 focus:ring-indigo-500 ${
                      errors.name ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : ''
                    }`}
                    required
                  />
                </div>
                {errors.name && (
                  <p className="text-sm text-red-400">{errors.name}</p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="email" className="text-gray-300">
                  {t('register.email')}
                </Label>
                <div className="relative">
                  <Mail className="absolute left-3 top-3 h-4 w-4 text-gray-500" />
                  <Input
                    id="email"
                    name="email"
                    type="email"
                    placeholder={t('register.emailPlaceholder')}
                    value={formData.email}
                    onChange={handleChange}
                    className={`pl-10 bg-gray-800 border-gray-700 text-white placeholder-gray-500 focus:border-indigo-500 focus:ring-indigo-500 ${
                      errors.email ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : ''
                    }`}
                    required
                  />
                </div>
                {errors.email && (
                  <p className="text-sm text-red-400">{errors.email}</p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="password" className="text-gray-300">
                  {t('register.password')}
                </Label>
                <div className="relative">
                  <Lock className="absolute left-3 top-3 h-4 w-4 text-gray-500" />
                  <Input
                    id="password"
                    name="password"
                    type={showPassword ? 'text' : 'password'}
                    placeholder={t('register.passwordPlaceholder')}
                    value={formData.password}
                    onChange={handleChange}
                    className={`pl-10 pr-10 bg-gray-800 border-gray-700 text-white placeholder-gray-500 focus:border-indigo-500 focus:ring-indigo-500 ${
                      errors.password ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : ''
                    }`}
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
                {errors.password && (
                  <p className="text-sm text-red-400">{errors.password}</p>
                )}
              </div>

              <div className="space-y-2">
                <Label htmlFor="confirmPassword" className="text-gray-300">
                  {t('register.confirmPassword')}
                </Label>
                <div className="relative">
                  <Lock className="absolute left-3 top-3 h-4 w-4 text-gray-500" />
                  <Input
                    id="confirmPassword"
                    name="confirmPassword"
                    type={showConfirmPassword ? 'text' : 'password'}
                    placeholder={t('register.confirmPasswordPlaceholder')}
                    value={formData.confirmPassword}
                    onChange={handleChange}
                    className={`pl-10 pr-10 bg-gray-800 border-gray-700 text-white placeholder-gray-500 focus:border-indigo-500 focus:ring-indigo-500 ${
                      errors.confirmPassword ? 'border-red-500 focus:border-red-500 focus:ring-red-500' : ''
                    }`}
                    required
                  />
                  <button
                    type="button"
                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                    className="absolute right-3 top-3 text-gray-500 hover:text-gray-300"
                  >
                    {showConfirmPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  </button>
                </div>
                {errors.confirmPassword && (
                  <p className="text-sm text-red-400">{errors.confirmPassword}</p>
                )}
              </div>

              <Button
                type="submit"
                className="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-medium py-2.5"
                disabled={loading || Object.keys(errors).length > 0}
              >
                {loading ? (
                  <div className="flex items-center space-x-2">
                    <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white"></div>
                    <span>{t('register.creatingAccount')}</span>
                  </div>
                ) : (
                  t('register.createAccount')
                )}
              </Button>
            </form>

            <div className="text-center pt-4">
              <p className="text-gray-400 text-sm">
                {t('register.alreadyHaveAccount')}{' '}
                <Link 
                  to="/login" 
                  className="text-indigo-400 hover:text-indigo-300 font-medium transition-colors duration-200"
                >
                  {t('register.loginHere')}
                </Link>
              </p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};

export default Register;
