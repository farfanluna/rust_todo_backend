import React from 'react';
import { useTranslation } from 'react-i18next';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Globe } from 'lucide-react';

const TopBar = () => {
  const { t, i18n } = useTranslation();

  return (
    <div className="fixed top-0 left-0 right-0 bg-gray-900 border-b border-gray-800 h-16 flex items-center justify-between px-4 md:hidden">
      <h1 className="text-2xl font-bold text-white">{t('dashboard.title')}</h1>
      <Select onValueChange={(value) => i18n.changeLanguage(value)} value={i18n.language}>
        <SelectTrigger className="w-auto bg-gray-800 border-gray-700 text-white">
          <Globe className="w-4 h-4" />
        </SelectTrigger>
        <SelectContent>
          <SelectItem value="en">English</SelectItem>
          <SelectItem value="es">Español</SelectItem>
        </SelectContent>
      </Select>
    </div>
  );
};

export default TopBar;
