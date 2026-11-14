require "test_helper"

class UserTest < ActiveSupport::TestCase
  def valid_attributes
    { email_address: "admin@example.com", password: "supersecret" }
  end

  test "valid with an email and a long enough password" do
    assert User.new(valid_attributes).valid?
  end

  test "requires a unique email address" do
    User.create!(valid_attributes)
    duplicate = User.new(valid_attributes.merge(password: "anotherpassword"))

    assert_not duplicate.valid?
    assert duplicate.errors.of_kind?(:email_address, :taken)
  end

  test "normalizes email address to a stripped, downcased form" do
    user = User.create!(valid_attributes.merge(email_address: "  Admin@Example.com  "))
    assert_equal "admin@example.com", user.email_address
  end

  test "requires a password of at least 8 characters" do
    user = User.new(valid_attributes.merge(password: "short"))
    assert_not user.valid?
    assert user.errors.of_kind?(:password, :too_short)
  end

  test "authenticate succeeds only with the correct password" do
    user = User.create!(valid_attributes)

    assert user.authenticate("supersecret")
    assert_not user.authenticate("wrongpassword")
  end

  test "admin? reflects the role column" do
    admin = User.create!(valid_attributes.merge(role: "admin"))
    customer = User.create!(valid_attributes.merge(email_address: "customer@example.com", role: "customer"))

    assert admin.admin?
    assert_not customer.admin?
  end

  test "phone_prefix and phone_number split a stored phone number" do
    user = User.new(phone: "+43 6601234567")
    assert_equal "+43", user.phone_prefix
    assert_equal "6601234567", user.phone_number

    user = User.new(phone: "+420 777888999")
    assert_equal "+420", user.phone_prefix
    assert_equal "777888999", user.phone_number
  end

  test "phone_prefix falls back to the Czech prefix when the number has none" do
    user = User.new(phone: "777888999")
    assert_equal "+420", user.phone_prefix
    assert_equal "777888999", user.phone_number

    user = User.new(phone: nil)
    assert_equal "+420", user.phone_prefix
    assert_equal "", user.phone_number
  end
end
