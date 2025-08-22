import React from 'react';
import { useAuth } from '../App';
import { Button } from './ui/button';
import { Home, Shield, LogOut } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { NavLink } from 'react-router-dom';

const BottomNavBar = () => {
  const { t } = useTranslation();
  const { isAdmin, logout } = useAuth();

  const navLinkClasses = ({ isActive }) =>
    `flex flex-col items-center justify-center h-16 w-full text-xs transition-colors ${
      isActive ? 'text-indigo-400 bg-gray-800' : 'text-gray-400 hover:bg-gray-800'
    }`;

  return (
    <div className="fixed bottom-0 left-0 right-0 bg-gray-900 border-t border-gray-800 md:hidden">
      <div className="flex justify-around items-center h-16">
        <NavLink to="/dashboard" className={navLinkClasses}>
          <Home className="w-6 h-6 mb-1" />
          <span>{t('dashboard.title')}</span>
        </NavLink>
        {isAdmin && (
          <NavLink to="/admin" className={navLinkClasses}>
            <Shield className="w-6 h-6 mb-1" />
            <span className="text-center">{t('dashboard.adminPanel')}</span>
          </NavLink>
        )}
        <Button
          variant="ghost"
          onClick={logout}
          className="flex flex-col items-center justify-center h-16 w-full text-xs text-gray-400 hover:bg-gray-800 hover:text-indigo-400 rounded-none"
        >
          <LogOut className="w-6 h-6 mb-1" />
          <span>{t('dashboard.logout')}</span>
        </Button>
      </div>
    </div>
  );
};

export default BottomNavBar;
