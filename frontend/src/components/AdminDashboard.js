import React, { useState, useEffect } from 'react';
import { useAuth } from '../App';
import { useToast } from '../hooks/use-toast';
import axios from 'axios';
import { Button } from './ui/button';
import { Card, CardHeader, CardContent, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from './ui/tabs';
import {
  Users, CheckSquare, Clock, TrendingUp, BarChart3, 
  PieChart, Activity, ArrowLeft, Settings, UserCheck,
  AlertTriangle, Calendar, Target, MoreVertical, Shield, UserX, Globe
} from 'lucide-react';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from './ui/dropdown-menu';
import { useTranslation } from 'react-i18next';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import BottomNavBar from './BottomNavBar';
import TopBar from './TopBar';

const AdminDashboard = () => {
  const { t, i18n } = useTranslation();
  const { user, logout } = useAuth();
  const { toast } = useToast();
  const [stats, setStats] = useState(null);
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [usersLoading, setUsersLoading] = useState(false);
  const [pagination, setPagination] = useState({ page: 1, per_page: 10, total: 0, total_pages: 0 });

  useEffect(() => {
    loadStats();
    loadUsers();
  }, []);

  useEffect(() => {
    loadUsers();
  }, [pagination.page]);

  const loadStats = async () => {
    try {
      const response = await axios.get('/admin/stats');
      setStats(response.data);
    } catch (error) {
      console.error('Error loading stats:', error);
      toast({
        title: t('adminDashboard.error'),
        description: t('adminDashboard.errorLoadingStats'),
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  };

  const loadUsers = async () => {
    try {
      setUsersLoading(true);
      const response = await axios.get(`/admin/users?page=${pagination.page}&per_page=${pagination.per_page}`);
      setUsers(response.data.users);
      setPagination(response.data.pagination);
    } catch (error) {
      console.error('Error loading users:', error);
      toast({
        title: t('adminDashboard.error'),
        description: t('adminDashboard.errorLoadingUsers'),
        variant: "destructive",
      });
    } finally {
      setUsersLoading(false);
    }
  };

  const handleRoleChange = async (userId, newRole) => {
    try {
      await axios.put(`/admin/users/${userId}/role`, { role: newRole });
      toast({
        title: t('adminDashboard.success'),
        description: t('adminDashboard.roleUpdated', { role: newRole }),
      });
      loadUsers(); // Recargar la lista de usuarios
    } catch (error) {
      console.error('Error updating user role:', error);
      toast({
        title: t('adminDashboard.error'),
        description: t('adminDashboard.errorUpdatingRole'),
        variant: "destructive",
      });
    }
  };

  const StatCard = ({ title, value, icon: Icon, description, color = "bg-indigo-600" }) => (
    <Card className="bg-gray-900 border-gray-800 hover:border-gray-700 transition-colors">
      <CardContent className="p-6">
        <div className="flex items-center justify-between">
          <div>
            <p className="text-gray-400 text-sm font-medium">{title}</p>
            <p className="text-3xl font-bold text-white mt-2">{value}</p>
            {description && (
              <p className="text-gray-500 text-sm mt-1">{description}</p>
            )}
          </div>
          <div className={`p-3 rounded-full ${color}`}>
            <Icon className="w-6 h-6 text-white" />
          </div>
        </div>
      </CardContent>
    </Card>
  );

  const ProgressBar = ({ label, value, total, color = "bg-indigo-500" }) => {
    const percentage = total > 0 ? (value / total) * 100 : 0;
    return (
      <div className="space-y-2">
        <div className="flex justify-between text-sm">
          <span className="text-gray-300">{label}</span>
          <span className="text-gray-400">{value}</span>
        </div>
        <div className="w-full bg-gray-700 rounded-full h-2">
          <div
            className={`${color} h-2 rounded-full transition-all duration-500`}
            style={{ width: `${percentage}%` }}
          ></div>
        </div>
      </div>
    );
  };

  if (loading) {
    return (
      <div className="min-h-screen bg-gray-950 flex items-center justify-center">
        <div className="text-center">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-indigo-500 mx-auto"></div>
          <p className="text-gray-400 mt-4">{t('adminDashboard.loadingStats')}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="min-h-screen bg-gray-950 pt-16 md:pt-0 pb-16 md:pb-0">
      <TopBar />
      {/* Header */}
      <header className="bg-gray-900 border-b border-gray-800 hidden md:block">
        <div className="max-w-full mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col md:flex-row justify-between items-center py-4 md:h-16">
            <div className="flex items-center space-x-4 mb-4 md:mb-0">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => window.location.href = '/dashboard'}
                className="text-gray-400 hover:text-white"
              >
                <ArrowLeft className="w-4 h-4 mr-2" />
                {t('adminDashboard.backToDashboard')}
              </Button>
              <div className="h-6 w-px bg-gray-700"></div>
              <h1 className="text-2xl font-bold text-white">{t('adminDashboard.adminPanel')}</h1>
              <Badge className="bg-emerald-600 text-white">{t('adminDashboard.admin')}</Badge>
            </div>
            
            <div className="flex items-center space-x-2 md:space-x-4">
              <span className="text-gray-300 text-sm md:text-base">{t('adminDashboard.greeting', { name: user?.name })}</span>
              <Select onValueChange={(value) => i18n.changeLanguage(value)} value={i18n.language}>
                <SelectTrigger className="w-auto bg-gray-800 border-gray-700 text-white">
                  <Globe className="w-4 h-4" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="en">English</SelectItem>
                  <SelectItem value="es">Español</SelectItem>
                </SelectContent>
              </Select>
              <Button
                variant="outline"
                size="sm"
                onClick={logout}
                className="border-gray-700 text-gray-300 hover:bg-gray-800"
              >
                {t('adminDashboard.logout')}
              </Button>
            </div>
          </div>
        </div>
      </header>

      <div className="max-w-full mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <Tabs defaultValue="overview" className="space-y-6">
          <TabsList className="bg-gray-900 border-gray-800">
            <TabsTrigger value="overview" className="data-[state=active]:bg-gray-800">
              <BarChart3 className="w-4 h-4 mr-2" />
              {t('adminDashboard.overview')}
            </TabsTrigger>
            <TabsTrigger value="users" className="data-[state=active]:bg-gray-800">
              <Users className="w-4 h-4 mr-2" />
              {t('adminDashboard.userManagement')}
            </TabsTrigger>
          </TabsList>

          {/* Tab: Resumen General */}
          <TabsContent value="overview" className="space-y-6">
            {/* Estadísticas principales */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
              <StatCard
                title={t('adminDashboard.totalUsers')}
                value={stats?.total_users || 0}
                icon={Users}
                description={t('adminDashboard.registeredUsers')}
                color="bg-blue-600"
              />
              <StatCard
                title={t('adminDashboard.totalTasks')}
                value={stats?.total_tasks || 0}
                icon={CheckSquare}
                description={t('adminDashboard.tasksInSystem')}
                color="bg-indigo-600"
              />
              <StatCard
                title={t('adminDashboard.newUsersToday')}
                value={stats?.recent_activity?.new_users_today || 0}
                icon={UserCheck}
                description={t('adminDashboard.registrationsToday')}
                color="bg-emerald-600"
              />
              <StatCard
                title={t('adminDashboard.tasksCompletedToday')}
                value={stats?.recent_activity?.tasks_completed_today || 0}
                icon={Target}
                description={t('adminDashboard.finishedToday')}
                color="bg-green-600"
              />
            </div>

            <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
              {/* Distribución por estado */}
              <Card className="bg-gray-900 border-gray-800">
                <CardHeader>
                  <CardTitle className="text-white flex items-center">
                    <PieChart className="w-5 h-5 mr-2" />
                    {t('adminDashboard.tasksByStatus')}
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <ProgressBar
                    label={t('adminDashboard.todo')}
                    value={stats?.tasks_by_status?.todo || 0}
                    total={stats?.total_tasks || 1}
                    color="bg-gray-500"
                  />
                  <ProgressBar
                    label={t('adminDashboard.doing')}
                    value={stats?.tasks_by_status?.doing || 0}
                    total={stats?.total_tasks || 1}
                    color="bg-blue-500"
                  />
                  <ProgressBar
                    label={t('adminDashboard.done')}
                    value={stats?.tasks_by_status?.done || 0}
                    total={stats?.total_tasks || 1}
                    color="bg-green-500"
                  />
                </CardContent>
              </Card>

              {/* Distribución por prioridad */}
              <Card className="bg-gray-900 border-gray-800">
                <CardHeader>
                  <CardTitle className="text-white flex items-center">
                    <AlertTriangle className="w-5 h-5 mr-2" />
                    {t('adminDashboard.tasksByPriority')}
                  </CardTitle>
                </CardHeader>
                <CardContent className="space-y-4">
                  <ProgressBar
                    label={t('adminDashboard.highPriority')}
                    value={stats?.tasks_by_priority?.high || 0}
                    total={stats?.total_tasks || 1}
                    color="bg-red-500"
                  />
                  <ProgressBar
                    label={t('adminDashboard.mediumPriority')}
                    value={stats?.tasks_by_priority?.med || 0}
                    total={stats?.total_tasks || 1}
                    color="bg-yellow-500"
                  />
                  <ProgressBar
                    label={t('adminDashboard.lowPriority')}
                    value={stats?.tasks_by_priority?.low || 0}
                    total={stats?.total_tasks || 1}
                    color="bg-green-500"
                  />
                </CardContent>
              </Card>
            </div>

            {/* Actividad reciente */}
            <Card className="bg-gray-900 border-gray-800">
              <CardHeader>
                <CardTitle className="text-white flex items-center">
                  <Activity className="w-5 h-5 mr-2" />
                  {t('adminDashboard.recentActivity')}
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                  <div className="text-center p-4 bg-gray-800 rounded-lg">
                    <Calendar className="w-8 h-8 text-blue-400 mx-auto mb-2" />
                    <p className="text-2xl font-bold text-white">
                      {stats?.recent_activity?.new_users_today || 0}
                    </p>
                    <p className="text-gray-400 text-sm">{t('adminDashboard.newUsers')}</p>
                  </div>
                  <div className="text-center p-4 bg-gray-800 rounded-lg">
                    <Clock className="w-8 h-8 text-yellow-400 mx-auto mb-2" />
                    <p className="text-2xl font-bold text-white">
                      {stats?.recent_activity?.tasks_created_today || 0}
                    </p>
                    <p className="text-gray-400 text-sm">{t('adminDashboard.tasksCreated')}</p>
                  </div>
                  <div className="text-center p-4 bg-gray-800 rounded-lg">
                    <CheckSquare className="w-8 h-8 text-green-400 mx-auto mb-2" />
                    <p className="text-2xl font-bold text-white">
                      {stats?.recent_activity?.tasks_completed_today || 0}
                    </p>
                    <p className="text-gray-400 text-sm">{t('adminDashboard.tasksCompleted')}</p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </TabsContent>

          {/* Tab: Gestión de Usuarios */}
          <TabsContent value="users" className="space-y-6">
            <Card className="bg-gray-900 border-gray-800">
              <CardHeader className="flex flex-row items-center justify-between">
                <CardTitle className="text-white">{t('adminDashboard.userList')}</CardTitle>
                <Badge variant="outline" className="text-gray-400">
                  {t('adminDashboard.usersCount', { count: pagination.total })}
                </Badge>
              </CardHeader>
              <CardContent>
                {usersLoading ? (
                  <div className="space-y-4">
                    {[...Array(5)].map((_, i) => (
                      <div key={i} className="animate-pulse">
                        <div className="h-16 bg-gray-800 rounded-lg"></div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <>
                    <div className="space-y-4">
                      {users.map(user => (
                        <div key={user.id} className="flex flex-col md:flex-row items-start md:items-center justify-between p-4 bg-gray-800 rounded-lg hover:bg-gray-750 transition-colors">
                          <div className="flex items-center space-x-4 w-full md:w-auto mb-4 md:mb-0">
                            <div className="w-10 h-10 bg-indigo-600 rounded-full flex items-center justify-center flex-shrink-0">
                              <span className="text-white font-semibold text-sm">
                                {user.name.charAt(0).toUpperCase()}
                              </span>
                            </div>
                            <div className="flex-grow">
                              <p className="text-white font-medium">{user.name}</p>
                              <p className="text-gray-400 text-sm break-all">{user.email}</p>
                            </div>
                          </div>
                          <div className="flex items-center space-x-2 md:space-x-4 w-full md:w-auto justify-end">
                            <Badge
                              className={
                                user.role === 'admin' 
                                  ? 'bg-emerald-600 text-white' 
                                  : 'bg-gray-600 text-gray-300'
                              }
                            >
                              {user.role === 'admin' ? t('adminDashboard.administrator') : t('adminDashboard.user')}
                            </Badge>
                            <div className="text-right text-xs md:text-sm">
                              <p className="text-gray-300">{t('adminDashboard.tasksCount', { count: user.task_count })}</p>
                              <p className="text-gray-500">
                                {t('adminDashboard.since', { date: new Date(user.created_at).toLocaleDateString() })}
                              </p>
                            </div>
                            <DropdownMenu>
                              <DropdownMenuTrigger asChild>
                                <Button variant="ghost" size="icon" className="text-gray-400 hover:text-white">
                                  <MoreVertical className="w-4 h-4" />
                                </Button>
                              </DropdownMenuTrigger>
                              <DropdownMenuContent className="bg-gray-800 border-gray-700 text-white">
                                {user.role !== 'admin' ? (
                                  <DropdownMenuItem onClick={() => handleRoleChange(user.id, 'admin')}>
                                    <Shield className="w-4 h-4 mr-2" />
                                    {t('adminDashboard.makeAdmin')}
                                  </DropdownMenuItem>
                                ) : (
                                  <DropdownMenuItem onClick={() => handleRoleChange(user.id, 'user')}>
                                    <UserX className="w-4 h-4 mr-2" />
                                    {t('adminDashboard.removeAdmin')}
                                  </DropdownMenuItem>
                                )}
                              </DropdownMenuContent>
                            </DropdownMenu>
                          </div>
                        </div>
                      ))}
                    </div>

                    {/* Paginación de usuarios */}
                    {pagination.total_pages > 1 && (
                      <div className="flex justify-center items-center space-x-2 mt-6 pt-6 border-t border-gray-800">
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => setPagination(prev => ({ ...prev, page: Math.max(1, prev.page - 1) }))}
                          disabled={pagination.page === 1}
                          className="border-gray-700 text-gray-300"
                        >
                          {t('adminDashboard.previous')}
                        </Button>
                        <span className="text-gray-400 px-4">
                          {t('adminDashboard.page', { page: pagination.page, totalPages: pagination.total_pages })}
                        </span>
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => setPagination(prev => ({ ...prev, page: Math.min(prev.total_pages, prev.page + 1) }))}
                          disabled={pagination.page === pagination.total_pages}
                          className="border-gray-700 text-gray-300"
                        >
                          {t('adminDashboard.next')}
                        </Button>
                      </div>
                    )}
                  </>
                )}
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </div>
      <BottomNavBar />
    </div>
  );
};

export default AdminDashboard;
