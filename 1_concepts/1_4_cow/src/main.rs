use std::borrow::Cow;
use std::env;

/// Определяет путь к конфигурационному файлу с учетом приоритетов:
/// 1. --conf аргумент командной строки (высший приоритет)
/// 2. APP_CONF переменная окружения
/// 3. /etc/app/app.conf (по умолчанию)
/// 
/// Возвращает Cow<str> для эффективного управления памятью:
/// - Cow::Borrowed для статических строк (без аллокации)
/// - Cow::Owned для динамически созданных строк
fn get_config_path() -> Result<Cow<'static, str>, String> {
    // Проверяем аргументы командной строки
    let args: Vec<String> = env::args().collect();
    
    // Ищем --conf аргумент
    for (i, arg) in args.iter().enumerate() {
        if arg == "--conf" {
            if i + 1 < args.len() {
                let conf_path = &args[i + 1];
                if conf_path.is_empty() {
                    return Err("Error: --conf argument cannot be empty".to_string());
                }
                // Возвращаем owned String, так как это динамически созданная строка
                return Ok(Cow::Owned(conf_path.clone()));
            } else {
                return Err("Error: --conf argument requires a value".to_string());
            }
        }
    }
    
    // Проверяем переменную окружения APP_CONF
    if let Ok(env_conf) = env::var("APP_CONF") {
        if !env_conf.is_empty() {
            // Возвращаем owned String, так как это значение из переменной окружения
            return Ok(Cow::Owned(env_conf));
        }
    }
    
    // Возвращаем путь по умолчанию как статическую строку (без аллокации)
    Ok(Cow::Borrowed("/etc/app/app.conf"))
}

/// Демонстрирует различные способы использования Cow<str>
fn demonstrate_cow_usage() {
    println!("\n=== Cow<str> Usage Examples ===");
    
    // Пример 1: Статическая строка (без аллокации)
    let static_cow: Cow<'static, str> = Cow::Borrowed("static string");
    println!("Static Cow: {}", static_cow);
    
    // Пример 2: Динамическая строка (с аллокацией)
    let dynamic_cow: Cow<'static, str> = Cow::Owned("dynamic string".to_string());
    println!("Dynamic Cow: {}", dynamic_cow);
    
    // Пример 3: Преобразование из &str в Cow
    let borrowed_str = "borrowed";
    let cow_from_str: Cow<'static, str> = borrowed_str.into();
    println!("Cow from &str: {}", cow_from_str);
    
    // Пример 4: Преобразование из String в Cow
    let owned_string = String::from("owned");
    let cow_from_string: Cow<'static, str> = owned_string.into();
    println!("Cow from String: {}", cow_from_string);
    
    // Пример 5: Условное создание Cow
    let use_owned = true;
    let conditional_cow: Cow<'static, str> = if use_owned {
        Cow::Owned("conditional owned".to_string())
    } else {
        Cow::Borrowed("conditional borrowed")
    };
    println!("Conditional Cow: {}", conditional_cow);
    
    // Пример 6: Клонирование Cow (ленивое клонирование)
    let original_cow: Cow<'static, str> = Cow::Borrowed("original");
    let cloned_cow = original_cow.clone();
    println!("Original: {}, Cloned: {}", original_cow, cloned_cow);
    
    // Пример 7: Мутация Cow (только для owned)
    let mut mutable_cow: Cow<'static, str> = Cow::Owned("mutable".to_string());
    mutable_cow.to_mut().push_str(" - modified");
    println!("Mutable Cow: {}", mutable_cow);
}

fn main() {
    println!("Configuration Path Detector");
    println!("============================");
    
    // Получаем путь к конфигурационному файлу
    match get_config_path() {
        Ok(path) => {
            println!("Configuration file path: {}", path);
            
            // Демонстрируем, что Cow эффективно управляет памятью
            match path {
                Cow::Borrowed(static_path) => {
                    println!("✓ Using static path (no allocation): {}", static_path);
                }
                Cow::Owned(owned_path) => {
                    println!("✓ Using owned path (allocation occurred): {}", owned_path);
                }
            }
        }
        Err(error) => {
            eprintln!("Error: {}", error);
            std::process::exit(1);
        }
    }
    
    // Демонстрируем различные способы использования Cow<str>
    demonstrate_cow_usage();
    
    println!("\n=== Memory Efficiency Test ===");
    
    // Тестируем эффективность памяти
    let default_path = get_config_path().unwrap();
    println!("Default path type: {}", 
        match default_path {
            Cow::Borrowed(_) => "Borrowed (no allocation)",
            Cow::Owned(_) => "Owned (allocation occurred)",
        }
    );
    
    // Показываем, что можно работать с Cow как с обычной строкой
    println!("Path length: {}", default_path.len());
    println!("Path starts with '/etc': {}", default_path.starts_with("/etc"));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// Вспомогательная функция для тестирования с заданными аргументами командной строки
    fn test_get_config_path_with_args(args: Vec<&str>) -> Result<Cow<'static, str>, String> {
        // Проверяем аргументы командной строки
        for (i, arg) in args.iter().enumerate() {
            if *arg == "--conf" {
                if i + 1 < args.len() {
                    let conf_path = args[i + 1];
                    if conf_path.is_empty() {
                        return Err("Error: --conf argument cannot be empty".to_string());
                    }
                    return Ok(Cow::Owned(conf_path.to_string()));
                } else {
                    return Err("Error: --conf argument requires a value".to_string());
                }
            }
        }
        
        // Проверяем переменную окружения APP_CONF
        if let Ok(env_conf) = env::var("APP_CONF") {
            if !env_conf.is_empty() {
                return Ok(Cow::Owned(env_conf));
            }
        }
        
        // Возвращаем путь по умолчанию
        Ok(Cow::Borrowed("/etc/app/app.conf"))
    }

    #[test]
    fn test_default_path() {
        // Очищаем переменные окружения для чистого теста
        unsafe { env::remove_var("APP_CONF"); }
        
        let result = test_get_config_path_with_args(vec!["program_name"]);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        match path {
            Cow::Borrowed(p) => assert_eq!(p, "/etc/app/app.conf"),
            Cow::Owned(_) => panic!("Expected borrowed path for default"),
        }
    }

    #[test]
    fn test_env_var_priority() {
        // Устанавливаем переменную окружения
        unsafe { env::set_var("APP_CONF", "/custom/path.conf"); }
        
        let result = test_get_config_path_with_args(vec!["program_name"]);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        match path {
            Cow::Owned(p) => assert_eq!(p, "/custom/path.conf"),
            Cow::Borrowed(_) => panic!("Expected owned path for env var"),
        }
        
        // Очищаем переменную окружения
        unsafe { env::remove_var("APP_CONF"); }
    }

    #[test]
    fn test_command_line_priority() {
        let result = test_get_config_path_with_args(vec!["program_name", "--conf", "/cli/path.conf"]);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        match path {
            Cow::Owned(p) => assert_eq!(p, "/cli/path.conf"),
            Cow::Borrowed(_) => panic!("Expected owned path for CLI arg"),
        }
    }

    #[test]
    fn test_empty_conf_argument() {
        let result = test_get_config_path_with_args(vec!["program_name", "--conf"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("requires a value"));
    }

    #[test]
    fn test_empty_conf_value() {
        let result = test_get_config_path_with_args(vec!["program_name", "--conf", ""]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("cannot be empty"));
    }

    #[test]
    fn test_priority_order() {
        // Устанавливаем переменную окружения
        unsafe { env::set_var("APP_CONF", "/env/path.conf"); }
        
        // CLI аргумент должен иметь приоритет над переменной окружения
        let result = test_get_config_path_with_args(vec!["program_name", "--conf", "/cli/path.conf"]);
        assert!(result.is_ok());
        
        let path = result.unwrap();
        match path {
            Cow::Owned(p) => assert_eq!(p, "/cli/path.conf"),
            Cow::Borrowed(_) => panic!("Expected owned path for CLI arg"),
        }
        
        // Очищаем переменную окружения
        unsafe { env::remove_var("APP_CONF"); }
    }

    #[test]
    fn test_cow_memory_efficiency() {
        // Тест для проверки эффективности памяти Cow
        let static_cow: Cow<'static, str> = Cow::Borrowed("static");
        let owned_cow: Cow<'static, str> = Cow::Owned("owned".to_string());
        
        // Проверяем, что статическая строка не требует аллокации
        match static_cow {
            Cow::Borrowed(_) => assert!(true), // Ожидаем Borrowed
            Cow::Owned(_) => panic!("Static string should be borrowed"),
        }
        
        // Проверяем, что owned строка требует аллокации
        match owned_cow {
            Cow::Owned(_) => assert!(true), // Ожидаем Owned
            Cow::Borrowed(_) => panic!("Owned string should be owned"),
        }
    }

    #[test]
    fn test_cow_cloning() {
        // Тест для проверки ленивого клонирования Cow
        let original: Cow<'static, str> = Cow::Borrowed("original");
        let cloned = original.clone();
        
        // Оба должны быть одинаковыми
        assert_eq!(original, cloned);
        
        // Для borrowed строк клонирование должно быть дешевым
        match (original, cloned) {
            (Cow::Borrowed(a), Cow::Borrowed(b)) => {
                assert_eq!(a, b);
                // Проверяем, что это один и тот же указатель (для статических строк)
                assert_eq!(a.as_ptr(), b.as_ptr());
            }
            _ => panic!("Expected both to be borrowed"),
        }
    }

    #[test]
    fn test_cow_mutation() {
        // Тест для проверки мутации Cow
        let mut owned_cow: Cow<'static, str> = Cow::Owned("mutable".to_string());
        
        // Мутация должна работать только для owned
        owned_cow.to_mut().push_str(" - modified");
        assert_eq!(owned_cow, "mutable - modified");
        
        // Попытка мутации borrowed должна создать owned копию
        let mut borrowed_cow: Cow<'static, str> = Cow::Borrowed("immutable");
        borrowed_cow.to_mut().push_str(" - now mutable");
        assert_eq!(borrowed_cow, "immutable - now mutable");
        
        // После мутации borrowed становится owned
        match borrowed_cow {
            Cow::Owned(_) => assert!(true),
            Cow::Borrowed(_) => panic!("Borrowed should become owned after mutation"),
        }
    }
}
