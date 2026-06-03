/**
 * Exodus Browser — profile management settings UI strings.
 */
import { resolveAppLocale, type AppLocale } from './appLocale';

export interface ProfileSettingsStrings {
  title: string;
  hint: string;
  currentProfile: string;
  profiles: string;
  edit: string;
  switch: string;
  delete: string;
  createProfile: string;
  guestProfile: string;
  guestProfileHint: string;
  noEmail: string;
  saved: string;
  saveError: string;
  profileCreated: string;
  profileSwitched: string;
  profileDeleted: string;
  profileUpdated: string;
  enterProfileName: string;
  loading: string;
}

const EN: ProfileSettingsStrings = {
  title: 'Profile management',
  hint: 'Manage your browser profiles, create new ones, or switch between them.',
  currentProfile: 'Current profile',
  profiles: 'All profiles',
  edit: 'Edit',
  switch: 'Switch',
  delete: 'Delete',
  createProfile: 'Create new profile',
  guestProfile: 'Enable guest profile',
  guestProfileHint: 'Guest profile allows browsing without saving data to your main profile.',
  noEmail: 'No email',
  saved: 'Profile settings saved',
  saveError: 'Failed to save profile settings',
  profileCreated: 'Profile created',
  profileSwitched: 'Profile switched',
  profileDeleted: 'Profile deleted',
  profileUpdated: 'Profile updated',
  enterProfileName: 'Enter profile name:',
  loading: 'Loading...',
};

const ZH: ProfileSettingsStrings = {
  title: '配置文件管理',
  hint: '管理浏览器配置文件，创建新配置文件或在配置文件之间切换。',
  currentProfile: '当前配置文件',
  profiles: '所有配置文件',
  edit: '编辑',
  switch: '切换',
  delete: '删除',
  createProfile: '创建新配置文件',
  guestProfile: '启用访客配置文件',
  guestProfileHint: '访客配置文件允许在不将数据保存到主配置文件的情况下浏览。',
  noEmail: '无邮箱',
  saved: '配置文件设置已保存',
  saveError: '保存配置文件设置失败',
  profileCreated: '配置文件已创建',
  profileSwitched: '配置文件已切换',
  profileDeleted: '配置文件已删除',
  profileUpdated: '配置文件已更新',
  enterProfileName: '输入配置文件名称：',
  loading: '加载中...',
};

const JA: ProfileSettingsStrings = {
  title: 'プロファイル管理',
  hint: 'ブラウザのプロファイルを管理し、新しいプロファイルを作成したり切り替えたりします。',
  currentProfile: '現在のプロファイル',
  profiles: 'すべてのプロファイル',
  edit: '編集',
  switch: '切り替え',
  delete: '削除',
  createProfile: '新しいプロファイルを作成',
  guestProfile: 'ゲストプロファイルを有効にする',
  guestProfileHint: 'ゲストプロファイルでは、メインプロファイルにデータを保存せずにブラウジングできます。',
  noEmail: 'メールなし',
  saved: 'プロファイル設定が保存されました',
  saveError: 'プロファイル設定の保存に失敗しました',
  profileCreated: 'プロファイルが作成されました',
  profileSwitched: 'プロファイルが切り替わりました',
  profileDeleted: 'プロファイルが削除されました',
  profileUpdated: 'プロファイルが更新されました',
  enterProfileName: 'プロファイル名を入力：',
  loading: '読み込み中...',
};

const KO: ProfileSettingsStrings = {
  title: '프로필 관리',
  hint: '브라우저 프로필을 관리하고, 새 프로필을 만들거나 프로필 간에 전환합니다.',
  currentProfile: '현재 프로필',
  profiles: '모든 프로필',
  edit: '편집',
  switch: '전환',
  delete: '삭제',
  createProfile: '새 프로필 만들기',
  guestProfile: '게스트 프로필 사용',
  guestProfileHint: '게스트 프로필을 사용하면 기본 프로필에 데이터를 저장하지 않고 브라우징할 수 있습니다.',
  noEmail: '이메일 없음',
  saved: '프로필 설정이 저장되었습니다',
  saveError: '프로필 설정 저장 실패',
  profileCreated: '프로필이 생성되었습니다',
  profileSwitched: '프로필이 전환되었습니다',
  profileDeleted: '프로필이 삭제되었습니다',
  profileUpdated: '프로필이 업데이트되었습니다',
  enterProfileName: '프로필 이름 입력:',
  loading: '로딩 중...',
};

const ES: ProfileSettingsStrings = {
  title: 'Gestión de perfiles',
  hint: 'Administra tus perfiles del navegador, crea nuevos o cambia entre ellos.',
  currentProfile: 'Perfil actual',
  profiles: 'Todos los perfiles',
  edit: 'Editar',
  switch: 'Cambiar',
  delete: 'Eliminar',
  createProfile: 'Crear nuevo perfil',
  guestProfile: 'Habilitar perfil de invitado',
  guestProfileHint: 'El perfil de invitado permite navegar sin guardar datos en tu perfil principal.',
  noEmail: 'Sin correo',
  saved: 'Configuración de perfil guardada',
  saveError: 'Error al guardar la configuración del perfil',
  profileCreated: 'Perfil creado',
  profileSwitched: 'Perfil cambiado',
  profileDeleted: 'Perfil eliminado',
  profileUpdated: 'Perfil actualizado',
  enterProfileName: 'Ingresa el nombre del perfil:',
  loading: 'Cargando...',
};

const FR: ProfileSettingsStrings = {
  title: 'Gestion des profils',
  hint: 'Gérez vos profils de navigateur, créez-en de nouveaux ou basculez entre eux.',
  currentProfile: 'Profil actuel',
  profiles: 'Tous les profils',
  edit: 'Modifier',
  switch: 'Basculer',
  delete: 'Supprimer',
  createProfile: 'Créer un nouveau profil',
  guestProfile: 'Activer le profil invité',
  guestProfileHint: 'Le profil invité permet de naviguer sans enregistrer de données dans votre profil principal.',
  noEmail: 'Sans e-mail',
  saved: 'Paramètres du profil enregistrés',
  saveError: 'Échec de l\'enregistrement des paramètres du profil',
  profileCreated: 'Profil créé',
  profileSwitched: 'Profil basculé',
  profileDeleted: 'Profil supprimé',
  profileUpdated: 'Profil mis à jour',
  enterProfileName: 'Entrez le nom du profil :',
  loading: 'Chargement...',
};

const DE: ProfileSettingsStrings = {
  title: 'Profilverwaltung',
  hint: 'Verwalten Sie Ihre Browserprofile, erstellen Sie neue oder wechseln Sie zwischen ihnen.',
  currentProfile: 'Aktuelles Profil',
  profiles: 'Alle Profile',
  edit: 'Bearbeiten',
  switch: 'Wechseln',
  delete: 'Löschen',
  createProfile: 'Neues Profil erstellen',
  guestProfile: 'Gastprofil aktivieren',
  guestProfileHint: 'Das Gastprofil ermöglicht das Surfen, ohne Daten im Hauptprofil zu speichern.',
  noEmail: 'Keine E-Mail',
  saved: 'Profileinstellungen gespeichert',
  saveError: 'Fehler beim Speichern der Profileinstellungen',
  profileCreated: 'Profil erstellt',
  profileSwitched: 'Profil gewechselt',
  profileDeleted: 'Profil gelöscht',
  profileUpdated: 'Profil aktualisiert',
  enterProfileName: 'Profilname eingeben:',
  loading: 'Wird geladen...',
};

const PT: ProfileSettingsStrings = {
  title: 'Gerenciamento de perfis',
  hint: 'Gerencie seus perfis do navegador, crie novos ou alterne entre eles.',
  currentProfile: 'Perfil atual',
  profiles: 'Todos os perfis',
  edit: 'Editar',
  switch: 'Alternar',
  delete: 'Excluir',
  createProfile: 'Criar novo perfil',
  guestProfile: 'Habilitar perfil de convidado',
  guestProfileHint: 'O perfil de convidado permite navegar sem salvar dados no seu perfil principal.',
  noEmail: 'Sem e-mail',
  saved: 'Configurações do perfil salvas',
  saveError: 'Falha ao salvar configurações do perfil',
  profileCreated: 'Perfil criado',
  profileSwitched: 'Perfil alternado',
  profileDeleted: 'Perfil excluído',
  profileUpdated: 'Perfil atualizado',
  enterProfileName: 'Digite o nome do perfil:',
  loading: 'Carregando...',
};

const RU: ProfileSettingsStrings = {
  title: 'Управление профилями',
  hint: 'Управляйте профилями браузера, создавайте новые или переключайтесь между ними.',
  currentProfile: 'Текущий профиль',
  profiles: 'Все профили',
  edit: 'Редактировать',
  switch: 'Переключить',
  delete: 'Удалить',
  createProfile: 'Создать новый профиль',
  guestProfile: 'Включить гостевой профиль',
  guestProfileHint: 'Гостевой профиль позволяет просматривать без сохранения данных в основном профиле.',
  noEmail: 'Нет электронной почты',
  saved: 'Настройки профиля сохранены',
  saveError: 'Не удалось сохранить настройки профиля',
  profileCreated: 'Профиль создан',
  profileSwitched: 'Профиль переключен',
  profileDeleted: 'Профиль удален',
  profileUpdated: 'Профиль обновлен',
  enterProfileName: 'Введите имя профиля:',
  loading: 'Загрузка...',
};

const PACKS: Record<AppLocale, ProfileSettingsStrings> = {
  en: EN,
  zh: ZH,
  ja: JA,
  ko: KO,
  es: ES,
  fr: FR,
  de: DE,
  pt: PT,
  ru: RU,
};

/** Localized profile settings copy. */
export function profileSettingsStrings(locale?: AppLocale): ProfileSettingsStrings {
  return PACKS[resolveAppLocale(locale)];
}
