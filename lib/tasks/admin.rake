namespace :admin do
  desc "Creates or updates the admin user from ADMIN_EMAIL / ADMIN_PASSWORD (see .env.example)"
  task create_user: :environment do
    email = ENV.fetch("ADMIN_EMAIL") { abort "Set ADMIN_EMAIL in your .env file first (see .env.example)." }
    password = ENV.fetch("ADMIN_PASSWORD") { abort "Set ADMIN_PASSWORD in your .env file first (see .env.example)." }

    user = User.find_or_initialize_by(email_address: email)
    user.password = password
    user.role = "admin"

    if user.save
      puts "Admin user ready: #{user.email_address}"
    else
      puts "Could not save admin user: #{user.errors.full_messages.join(', ')}"
    end
  end
end
