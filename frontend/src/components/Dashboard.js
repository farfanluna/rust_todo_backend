import React, { useState, useEffect, useCallback } from 'react';
import { useSearchParams } from 'react-router-dom';
import { useAuth } from '../App';
import { useToast } from '../hooks/use-toast';
import axios from 'axios';
import { Button } from './ui/button';
import { Input } from './ui/input';
import { Label } from './ui/label';
import { Card, CardHeader, CardContent, CardTitle } from './ui/card';
import { Badge } from './ui/badge';
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogHeader, DialogTitle, DialogTrigger } from './ui/dialog';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from './ui/select';
import { Textarea } from './ui/textarea';
import { Calendar } from './ui/calendar';
import { Popover, PopoverContent, PopoverTrigger } from './ui/popover';
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from './ui/accordion';
import {
  Search, Plus, Filter, Edit, Trash2, Calendar as CalendarIcon, 
  User, Users, ChevronDown, MoreHorizontal, Settings, LogOut,
  Clock, AlertCircle, CheckCircle, Circle, Play, Globe, Clipboard
} from 'lucide-react';
import { format } from 'date-fns';
import { es, enUS } from 'date-fns/locale';
import { useTranslation } from 'react-i18next';
import BottomNavBar from './BottomNavBar';
import TopBar from './TopBar';

const FilterContent = ({ filters, handleFilterChange, users, isAdmin }) => {
  const { t } = useTranslation();
  return (
    <div className="space-y-4 pt-4">
      {/* Búsqueda */}
      <div className="space-y-2">
        <Label className="text-gray-300">{t('dashboard.search')}</Label>
        <div className="relative">
          <Search className="absolute left-3 top-3 h-4 w-4 text-gray-500" />
          <Input
            placeholder={t('dashboard.searchTasks')}
            value={filters.search}
            onChange={(e) => handleFilterChange('search', e.target.value)}
            className="pl-10 bg-gray-800 border-gray-700 text-white"
          />
        </div>
      </div>

      {/* Estado */}
      <div className="space-y-2">
        <Label className="text-gray-300">{t('dashboard.status')}</Label>
        <Select
          value={filters.status}
          onValueChange={(value) => handleFilterChange('status', value)}
        >
          <SelectTrigger className="bg-gray-800 border-gray-700 text-white">
            <SelectValue placeholder={t('dashboard.allStatuses')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('dashboard.allStatuses')}</SelectItem>
            <SelectItem value="todo">{t('dashboard.todo')}</SelectItem>
            <SelectItem value="doing">{t('dashboard.doing')}</SelectItem>
            <SelectItem value="done">{t('dashboard.done')}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Prioridad */}
      <div className="space-y-2">
        <Label className="text-gray-300">{t('dashboard.priority')}</Label>
        <Select
          value={filters.priority}
          onValueChange={(value) => handleFilterChange('priority', value)}
        >
          <SelectTrigger className="bg-gray-800 border-gray-700 text-white">
            <SelectValue placeholder={t('dashboard.allPriorities')} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">{t('dashboard.allPriorities')}</SelectItem>
            <SelectItem value="high">{t('dashboard.high')}</SelectItem>
            <SelectItem value="med">{t('dashboard.medium')}</SelectItem>
            <SelectItem value="low">{t('dashboard.low')}</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {/* Tags */}
      <div className="space-y-2">
        <Label className="text-gray-300">{t('dashboard.tags')}</Label>
        <Input
          placeholder={t('dashboard.searchTags')}
          value={filters.tags}
          onChange={(e) => handleFilterChange('tags', e.target.value)}
          className="bg-gray-800 border-gray-700 text-white"
        />
      </div>

      {/* Asignado a (solo para admins) */}
      {isAdmin && (
        <div className="space-y-2">
          <Label className="text-gray-300">{t('dashboard.assignedTo')}</Label>
          <Select
            value={filters.assigned_to}
            onValueChange={(value) => handleFilterChange('assigned_to', value)}
          >
            <SelectTrigger className="bg-gray-800 border-gray-700 text-white">
              <SelectValue placeholder={t('dashboard.allUsers')} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t('dashboard.allUsers')}</SelectItem>
              {users.map(user => (
                <SelectItem key={user.id} value={user.name}>
                  {user.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>
      )}

      {/* Ordenar por */}
      <div className="space-y-2">
        <Label className="text-gray-300">{t('dashboard.sortBy')}</Label>
        <Select
          value={filters.sort_by}
          onValueChange={(value) => handleFilterChange('sort_by', value)}
        >
          <SelectTrigger className="bg-gray-800 border-gray-700 text-white">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="created_at">{t('dashboard.creationDate')}</SelectItem>
            <SelectItem value="updated_at">{t('dashboard.updateDate')}</SelectItem>
            <SelectItem value="due_date">{t('dashboard.dueDate')}</SelectItem>
            <SelectItem value="priority">{t('dashboard.priority')}</SelectItem>
            <SelectItem value="title">{t('dashboard.taskTitle')}</SelectItem>
          </SelectContent>
        </Select>
      </div>
    </div>
  );
};

const Dashboard = () => {
  const { t, i18n } = useTranslation();
  const { user, logout, isAdmin } = useAuth();
  const { toast } = useToast();
  const [tasks, setTasks] = useState([]);
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [searchParams, setSearchParams] = useSearchParams();

  // Estados para filtros y paginación derivados de la URL
  const page = parseInt(searchParams.get('page') || '1', 10);
  const per_page = parseInt(searchParams.get('per_page') || '10', 10);
  const filters = {
    search: searchParams.get('search') || '',
    status: searchParams.get('status') || '',
    priority: searchParams.get('priority') || '',
    tags: searchParams.get('tags') || '',
    due_date_start: searchParams.get('due_date_start') || '',
    due_date_end: searchParams.get('due_date_end') || '',
    assigned_to: searchParams.get('assigned_to') || '',
    sort_by: searchParams.get('sort_by') || 'created_at',
    sort_order: searchParams.get('sort_order') || 'desc'
  };

  const [pagination, setPagination] = useState({ page, per_page, total: 0, total_pages: 0 });
  const [statusCounts, setStatusCounts] = useState({ todo: 0, doing: 0, done: 0, total: 0 });

  // Estados para modal de tarea
  const [showTaskModal, setShowTaskModal] = useState(false);
  const [showFilterModal, setShowFilterModal] = useState(false);
  const [editingTask, setEditingTask] = useState(null);
  const [formErrors, setFormErrors] = useState({});
  const [taskForm, setTaskForm] = useState({
    title: '',
    description: '',
    status: 'todo',
    priority: 'med',
    due_date: null,
    tags: '',
    assigned_to: 'unassigned'
  });

  useEffect(() => {
    const errors = {};
    if (taskForm.title.trim().length > 0 && (taskForm.title.trim().length < 3 || taskForm.title.trim().length > 120)) {
      errors.title = t('dashboard.titleLengthError', { min: 3, max: 120 });
    } else if (taskForm.title.trim().length === 0) {
      errors.title = t('dashboard.titleRequiredError');
    }

    if (taskForm.due_date) {
      const today = new Date();
      today.setHours(0, 0, 0, 0);
      const selectedDate = new Date(taskForm.due_date);
      selectedDate.setHours(0, 0, 0, 0);

      if (selectedDate < today) {
        errors.due_date = t('dashboard.dueDateInPastError');
      }
    }
    
    setFormErrors(errors);
  }, [taskForm.title, taskForm.due_date, t]);

  const loadTasks = useCallback(async () => {
    try {
      setLoading(true);
      const params = new URLSearchParams(searchParams);
      
      const response = await axios.get(`/tasks?${params}`);
      setTasks(response.data.tasks);
      setPagination(response.data.pagination);
    } catch (error) {
      console.error('Error loading tasks:', error);
      toast({
        title: t('dashboard.error'),
        description: t('dashboard.errorLoadingTasks'),
        variant: "destructive",
      });
    } finally {
      setLoading(false);
    }
  }, [searchParams, toast]);

  // Cargar datos iniciales
  useEffect(() => {
    const handler = setTimeout(() => {
      loadTasks();
    }, 500); // Debounce de 500ms

    return () => {
      clearTimeout(handler);
    };
  }, [loadTasks]);

  useEffect(() => {
    loadUsers();
    loadStatusCounts();
  }, []);

  const loadStatusCounts = async () => {
    try {
      const response = await axios.get('/tasks/stats');
      const total = response.data.todo + response.data.doing + response.data.done;
      setStatusCounts({...response.data, total});
    } catch (error) {
      console.error('Error loading status counts:', error);
    }
  };

  const loadUsers = async () => {
    try {
      const response = await axios.get('/users');
      setUsers(response.data);
    } catch (error) {
      console.error('Error loading users:', error);
    }
  };

  const handleFilterChange = (key, value) => {
    const newParams = new URLSearchParams(searchParams);
    if (value && value !== 'all' && value !== 'unassigned') {
      newParams.set(key, value);
    } else {
      newParams.delete(key);
    }
    newParams.set('page', '1'); // Reset page on filter change
    setSearchParams(newParams);
  };

  const clearFilters = () => {
    setSearchParams({ page: '1', per_page: '10', sort_by: 'created_at', sort_order: 'desc' });
  };

  const openTaskModal = (task = null) => {
    if (task) {
      setEditingTask(task);
      setTaskForm({
        title: task.title,
        description: task.description || '',
        status: task.status,
        priority: task.priority,
        due_date: task.due_date ? new Date(task.due_date) : null,
        tags: task.tags || '',
        assigned_to: task.assigned_to || 'unassigned'
      });
    } else {
      setEditingTask(null);
      setTaskForm({
        title: '',
        description: '',
        status: 'todo',
        priority: 'med',
        due_date: null,
        tags: '',
        assigned_to: 'unassigned'
      });
    }
    setShowTaskModal(true);
  };

  const handleTaskSubmit = async (e) => {
    e.preventDefault();
    if (Object.keys(formErrors).length > 0) {
      toast({
        title: t('dashboard.invalidForm'),
        description: t('dashboard.fixErrors'),
        variant: "destructive",
      });
      return;
    }
    try {
      const taskData = {
        ...taskForm,
        due_date: taskForm.due_date ? format(taskForm.due_date, "yyyy-MM-dd'T'HH:mm:ss.SSS'Z'") : null,
        assigned_to: taskForm.assigned_to === 'unassigned' ? '' : taskForm.assigned_to
      };

      if (editingTask) {
        await axios.put(`/tasks/${editingTask.id}`, taskData);
        toast({
          title: t('dashboard.taskUpdated'),
          description: t('dashboard.taskUpdatedMessage'),
        });
      } else {
        await axios.post('/tasks', taskData);
        toast({
          title: t('dashboard.taskCreated'),
          description: t('dashboard.taskCreatedMessage'),
        });
      }

      setShowTaskModal(false);
      loadTasks();
      loadStatusCounts();
    } catch (error) {
      console.error('Error saving task:', error);
      toast({
        title: t('dashboard.error'),
        description: error.response?.data?.detail || t('dashboard.errorSavingTask'),
        variant: "destructive",
      });
    }
  };

  const handleCopyTask = (task) => {
    const taskText = `Title: ${task.title}\nDescription: ${task.description || ''}`;
    navigator.clipboard.writeText(taskText);
    toast({
      title: t('dashboard.taskCopied'),
      description: t('dashboard.taskCopiedMessage'),
    });
  };

  const deleteTask = async (taskId) => {
    if (!window.confirm(t('dashboard.deleteConfirmation'))) {
      return;
    }

    try {
      await axios.delete(`/tasks/${taskId}`);
      toast({
        title: t('dashboard.taskDeleted'),
        description: t('dashboard.taskDeletedMessage'),
      });
      loadTasks();
      loadStatusCounts();
    } catch (error) {
      console.error('Error deleting task:', error);
      toast({
        title: t('dashboard.error'),
        description: t('dashboard.errorDeletingTask'),
        variant: "destructive",
      });
    }
  };

  const getStatusIcon = (status) => {
    switch (status) {
      case 'todo': return <Circle className="w-4 h-4" />;
      case 'doing': return <Play className="w-4 h-4" />;
      case 'done': return <CheckCircle className="w-4 h-4" />;
      default: return <Circle className="w-4 h-4" />;
    }
  };

  const getPriorityColor = (priority) => {
    switch (priority) {
      case 'high': return 'bg-red-500/20 text-red-400 border-red-500/50';
      case 'med': return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/50';
      case 'low': return 'bg-green-500/20 text-green-400 border-green-500/50';
      default: return 'bg-gray-500/20 text-gray-400 border-gray-500/50';
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'todo': return 'bg-gray-500/20 text-gray-400 border-gray-500/50';
      case 'doing': return 'bg-blue-500/20 text-blue-400 border-blue-500/50';
      case 'done': return 'bg-green-500/20 text-green-400 border-green-500/50';
      default: return 'bg-gray-500/20 text-gray-400 border-gray-500/50';
    }
  };

  return (
    <div className="min-h-screen bg-gray-950 pt-16 md:pt-0 pb-16 md:pb-0">
      <TopBar />
      {/* Header */}
      <header className="bg-gray-900 border-b border-gray-800 hidden md:block">
        <div className="max-w-full mx-auto px-4 sm:px-6 lg:px-8">
          <div className="flex flex-col md:flex-row justify-between items-center py-4 md:h-16">
            <div className="flex items-center space-x-4 mb-4 md:mb-0">
              <h1 className="text-2xl font-bold text-white">{t('dashboard.title')}</h1>
              <Badge variant="outline" className="text-indigo-400 border-indigo-400">
                {isAdmin ? t('dashboard.admin') : t('dashboard.user')}
              </Badge>
            </div>
            
            <div className="flex items-center space-x-2 md:space-x-4">
              <span className="text-gray-300 text-sm md:text-base">{t('dashboard.greeting', { name: user?.name })}</span>
              <Select onValueChange={(value) => i18n.changeLanguage(value)} value={i18n.language}>
                <SelectTrigger className="w-auto bg-gray-800 border-gray-700 text-white">
                  <Globe className="w-4 h-4" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="en">English</SelectItem>
                  <SelectItem value="es">Español</SelectItem>
                </SelectContent>
              </Select>
              {isAdmin && (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => window.location.href = '/admin'}
                  className="border-gray-700 text-gray-300 hover:bg-gray-800"
                >
                  <Settings className="w-4 h-4 mr-2" />
                  {t('dashboard.adminPanel')}
                </Button>
              )}
              <Button
                variant="outline"
                size="sm"
                onClick={logout}
                className="border-gray-700 text-gray-300 hover:bg-gray-800"
              >
                <LogOut className="w-4 h-4 mr-2" />
                {t('dashboard.logout')}
              </Button>
            </div>
          </div>
        </div>
      </header>

      <div className="max-w-full mx-auto px-4 sm:px-6 lg:px-8 py-8">
        <div className="grid grid-cols-1 lg:grid-cols-4 gap-6">
          {/* Sidebar de filtros */}
          <div className="lg:col-span-1">
            <div className="lg:hidden mb-4">
              <Dialog open={showFilterModal} onOpenChange={setShowFilterModal}>
                <DialogTrigger asChild>
                  <Button variant="outline" className="w-full border-gray-700 text-gray-300">
                    <Filter className="w-4 h-4 mr-2" />
                    {t('dashboard.filters')}
                  </Button>
                </DialogTrigger>
                <DialogContent className="bg-gray-900 border-gray-800 text-white w-full max-w-[90vw] sm:max-w-sm">
                  <DialogHeader>
                    <DialogTitle>{t('dashboard.filters')}</DialogTitle>
                  </DialogHeader>
                  <FilterContent
                    filters={filters}
                    handleFilterChange={handleFilterChange}
                    users={users}
                    isAdmin={isAdmin}
                  />
                  <DialogFooter>
                    <Button onClick={() => setShowFilterModal(false)} className="w-full bg-indigo-600 hover:bg-indigo-700 text-white">
                      {t('dashboard.applyFilters')}
                    </Button>
                  </DialogFooter>
                </DialogContent>
              </Dialog>
            </div>
            <div className="hidden lg:block">
              <Card className="bg-gray-900 border-gray-800 sticky top-6">
                <CardHeader>
                  <CardTitle className="text-white flex items-center justify-between">
                    <span>{t('dashboard.filters')}</span>
                    <Button
                      variant="ghost"
                      size="sm"
                      onClick={clearFilters}
                      className="text-gray-400 hover:text-white"
                    >
                      {t('dashboard.clear')}
                    </Button>
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <FilterContent
                    filters={filters}
                    handleFilterChange={handleFilterChange}
                    users={users}
                    isAdmin={isAdmin}
                  />
                </CardContent>
              </Card>
            </div>
          </div>

          {/* Lista de tareas */}
          <div className="lg:col-span-3">
            {/* Task stats cards */}
            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-6">
              <Card className="bg-gray-900 border-gray-800">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium text-gray-400">{t('dashboard.totalTasks')}</CardTitle>
                  <Users className="h-4 w-4 text-gray-500" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-white">{statusCounts.total}</div>
                </CardContent>
              </Card>
              <Card className="bg-gray-900 border-gray-800">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium text-gray-400">{t('dashboard.todo')}</CardTitle>
                  <Circle className="h-4 w-4 text-gray-500" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-white">{statusCounts.todo}</div>
                </CardContent>
              </Card>
              <Card className="bg-gray-900 border-gray-800">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium text-gray-400">{t('dashboard.doing')}</CardTitle>
                  <Play className="h-4 w-4 text-blue-500" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-white">{statusCounts.doing}</div>
                </CardContent>
              </Card>
              <Card className="bg-gray-900 border-gray-800">
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium text-gray-400">{t('dashboard.done')}</CardTitle>
                  <CheckCircle className="h-4 w-4 text-green-500" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold text-white">{statusCounts.done}</div>
                </CardContent>
              </Card>
            </div>

            <div className="flex flex-col md:flex-row justify-between items-start md:items-center mb-6">
              <div className="mb-4 md:mb-0">
                <h2 className="text-2xl font-bold text-white">{t('dashboard.myTasks')}</h2>
              </div>
              <Button
                onClick={() => openTaskModal()}
                className="bg-indigo-600 hover:bg-indigo-700 text-white"
              >
                <Plus className="w-4 h-4 mr-2" />
                {t('dashboard.newTask')}
              </Button>
            </div>

            {loading ? (
              <div className="space-y-4">
                {[...Array(5)].map((_, i) => (
                  <Card key={i} className="bg-gray-900 border-gray-800">
                    <CardContent className="p-4">
                      <div className="animate-pulse">
                        <div className="h-4 bg-gray-700 rounded w-3/4 mb-2"></div>
                        <div className="h-3 bg-gray-700 rounded w-1/2"></div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            ) : tasks.length === 0 ? (
              <Card className="bg-gray-900 border-gray-800">
                <CardContent className="p-8 text-center">
                  <div className="text-gray-400">
                    <AlertCircle className="w-12 h-12 mx-auto mb-4 opacity-50" />
                    <p className="text-lg mb-2">{t('dashboard.noTasksFound')}</p>
                    <p className="text-sm">{t('dashboard.noTasksMessage')}</p>
                  </div>
                </CardContent>
              </Card>
            ) : (
              <div className="space-y-4">
                {tasks.map(task => (
                  <Card key={task.id} className="bg-gray-900 border-gray-800 hover:border-gray-700 transition-colors">
                    <CardContent className="p-6">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-start justify-between mb-2">
                            <div className="flex items-center space-x-3">
                              <div className={`p-1 rounded ${getStatusColor(task.status)}`}>
                                {getStatusIcon(task.status)}
                              </div>
                              <Badge className={getPriorityColor(task.priority)}>
                                {task.priority === 'high' ? t('dashboard.high') : task.priority === 'med' ? t('dashboard.medium') : t('dashboard.low')}
                              </Badge>
                            </div>
                            <div className="flex items-center space-x-2">
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => openTaskModal(task)}
                                className="text-gray-400 hover:text-white"
                              >
                                <Edit className="w-4 h-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => handleCopyTask(task)}
                                className="text-gray-400 hover:text-white"
                              >
                                <Clipboard className="w-4 h-4" />
                              </Button>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => deleteTask(task.id)}
                                className="text-gray-400 hover:text-red-400"
                              >
                                <Trash2 className="w-4 h-4" />
                              </Button>
                            </div>
                          </div>

                          <h3 className="text-lg font-semibold text-white min-w-0 mb-2">{task.title}</h3>
                          
                          {task.description && (
                            <p className="text-gray-400 mb-3">{task.description}</p>
                          )}
                          
                          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-sm text-gray-500">
                            {task.due_date && (
                              <div className="flex items-center space-x-1">
                                <CalendarIcon className="w-4 h-4" />
                                <span>
                                  {format(new Date(task.due_date), 'dd MMM yyyy', { locale: i18n.language === 'es' ? es : enUS })}
                                </span>
                              </div>
                            )}
                            {task.assigned_to && (
                              <div className="flex items-center space-x-1">
                                <User className="w-4 h-4" />
                                <span>{task.assigned_to}</span>
                              </div>
                            )}
                            {isAdmin && task.owner_name && (
                              <div className="flex items-center space-x-1">
                                <Users className="w-4 h-4" />
                                <span>{t('dashboard.owner', { name: task.owner_name })}</span>
                              </div>
                            )}
                          </div>
                          
                          {task.tags && (
                            <div className="mt-2">
                              <div className="flex flex-wrap gap-1">
                                {task.tags.split(',').map((tag, index) => (
                                  <Badge key={index} variant="outline" className="text-xs text-gray-400 border-gray-600">
                                    {tag.trim()}
                                  </Badge>
                                ))}
                              </div>
                            </div>
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
                
                {/* Paginación */}
                {pagination.total_pages > 1 && (
                  <div className="flex justify-center items-center space-x-2 mt-6">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const newParams = new URLSearchParams(searchParams);
                        newParams.set('page', Math.max(1, page - 1).toString());
                        setSearchParams(newParams);
                      }}
                      disabled={page === 1}
                      className="border-gray-700 text-gray-300"
                    >
                      {t('dashboard.previous')}
                    </Button>
                    <span className="text-gray-400 px-4">
                      {t('dashboard.page', { page, totalPages: pagination.total_pages })}
                    </span>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        const newParams = new URLSearchParams(searchParams);
                        newParams.set('page', Math.min(pagination.total_pages, page + 1).toString());
                        setSearchParams(newParams);
                      }}
                      disabled={page === pagination.total_pages}
                      className="border-gray-700 text-gray-300"
                    >
                      {t('dashboard.next')}
                    </Button>
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      </div>

      {/* Modal de tarea */}
      <Dialog open={showTaskModal} onOpenChange={setShowTaskModal}>
        <DialogContent className="bg-gray-900 border-gray-800 text-white w-full max-w-[90vw] sm:max-w-lg md:max-w-2xl rounded-md">
          <DialogHeader>
            <DialogTitle className="text-base">
              {editingTask ? t('dashboard.editTask') : t('dashboard.newTaskModalTitle')}
            </DialogTitle>
            <DialogDescription className="text-gray-400 text-xs">
              {editingTask ? t('dashboard.editTaskDescription') : t('dashboard.newTaskDescription')}
            </DialogDescription>
          </DialogHeader>
          
          <form onSubmit={handleTaskSubmit} className="space-y-1">
            <div className="flex flex-col gap-1">
              <div>
                <Label htmlFor="title" className="text-gray-300 text-xs">{t('dashboard.taskTitle')}</Label>
                <Input
                  id="title"
                  value={taskForm.title}
                  onChange={(e) => setTaskForm(prev => ({ ...prev, title: e.target.value }))}
                  className="bg-gray-800 border-gray-700 text-white h-8 text-sm"
                  required
                />
                {formErrors.title && <p className="text-red-500 text-xs mt-1">{formErrors.title}</p>}
              </div>
              
              <div>
                <Label htmlFor="description" className="text-gray-300 text-xs">{t('dashboard.description')}</Label>
                <Textarea
                  id="description"
                  value={taskForm.description}
                  onChange={(e) => setTaskForm(prev => ({ ...prev, description: e.target.value }))}
                  className="bg-gray-800 border-gray-700 text-white text-sm"
                  rows={2}
                />
              </div>
              
              <div>
                <Label className="text-gray-300 text-xs">{t('dashboard.status')}</Label>
                <Select
                  value={taskForm.status}
                  onValueChange={(value) => setTaskForm(prev => ({ ...prev, status: value }))}
                >
                  <SelectTrigger className="bg-gray-800 border-gray-700 text-white h-8 text-sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="todo">{t('dashboard.todo')}</SelectItem>
                    <SelectItem value="doing">{t('dashboard.doing')}</SelectItem>
                    <SelectItem value="done">{t('dashboard.done')}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label className="text-gray-300 text-xs">{t('dashboard.priority')}</Label>
                <Select
                  value={taskForm.priority}
                  onValueChange={(value) => setTaskForm(prev => ({ ...prev, priority: value }))}
                >
                  <SelectTrigger className="bg-gray-800 border-gray-700 text-white h-8 text-sm">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="high">{t('dashboard.high')}</SelectItem>
                    <SelectItem value="med">{t('dashboard.medium')}</SelectItem>
                    <SelectItem value="low">{t('dashboard.low')}</SelectItem>
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label className="text-gray-300 text-xs">{t('dashboard.dueDate')}</Label>
                <Popover>
                  <PopoverTrigger asChild>
                    <Button
                      variant="outline"
                      className="w-full justify-start text-left bg-gray-800 border-gray-700 text-white hover:bg-gray-700 h-8 text-sm"
                    >
                      <CalendarIcon className="mr-2 h-4 w-4" />
                      {taskForm.due_date ? format(taskForm.due_date, 'dd MMM yyyy', { locale: i18n.language === 'es' ? es : enUS }) : t('dashboard.selectDate')}
                    </Button>
                  </PopoverTrigger>
                  <PopoverContent className="w-auto p-0 bg-gray-800 border-gray-700">
                    <Calendar
                      mode="single"
                      selected={taskForm.due_date}
                      onSelect={(date) => setTaskForm(prev => ({ ...prev, due_date: date }))}
                      initialFocus
                      className="bg-gray-800"
                    />
                  </PopoverContent>
                </Popover>
                {formErrors.due_date && <p className="text-red-500 text-xs mt-1">{formErrors.due_date}</p>}
              </div>
              
              <div>
                <Label className="text-gray-300 text-xs">{t('dashboard.assignTo')}</Label>
                <Select
                  value={taskForm.assigned_to}
                  onValueChange={(value) => setTaskForm(prev => ({ ...prev, assigned_to: value }))}
                >
                  <SelectTrigger className="bg-gray-800 border-gray-700 text-white h-8 text-sm">
                    <SelectValue placeholder={t('dashboard.selectUser')} />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="unassigned">{t('dashboard.unassigned')}</SelectItem>
                    {users.map(user => (
                      <SelectItem key={user.id} value={user.name}>
                        {user.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
              
              <div>
                <Label htmlFor="tags" className="text-gray-300 text-xs">{t('dashboard.tags')}</Label>
                <Input
                  id="tags"
                  value={taskForm.tags}
                  onChange={(e) => setTaskForm(prev => ({ ...prev, tags: e.target.value }))}
                  className="bg-gray-800 border-gray-700 text-white h-8 text-sm"
                  placeholder={t('dashboard.tagsPlaceholder')}
                />
              </div>
            </div>
            
            <div className="flex justify-end space-x-2 pt-1">
              <Button
                type="button"
                variant="outline"
                onClick={() => setShowTaskModal(false)}
                className="border-gray-700 text-gray-300 h-8 text-xs"
              >
                {t('dashboard.cancel')}
              </Button>
              <Button
                type="submit"
                className="bg-indigo-600 hover:bg-indigo-700 text-white h-8 text-xs"
                disabled={Object.keys(formErrors).length > 0}
              >
                {editingTask ? t('dashboard.update') : t('dashboard.create')} {t('dashboard.task')}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
      <BottomNavBar />
    </div>
  );
};

export default Dashboard;
