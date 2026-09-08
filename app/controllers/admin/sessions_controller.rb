module Admin
  class SessionsController < BaseController
    layout "admin_login"
    allow_unauthenticated_access only: %i[new create]

    def new
    end

    def create
      user = User.find_by(email_address: params[:email_address].to_s.strip.downcase)

      if user&.authenticate(params[:password])
        start_new_session_for(user)
        redirect_to after_authentication_url
      else
        flash.now[:alert] = "Nesprávný e-mail nebo heslo."
        render :new, status: :unprocessable_entity
      end
    end

    def destroy
      terminate_session
      redirect_to new_admin_session_path
    end
  end
end
